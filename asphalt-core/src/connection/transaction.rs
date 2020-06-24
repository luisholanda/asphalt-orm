use super::RawConnection;
use crate::error::{Error, QueryResult};
use futures_util::future::{BoxFuture, CatchUnwind, TryFuture};
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Configuration of a transaction.
///
/// All the fields are optional to permit users to use the default value of the database.
/// Implementations of [`TransactionManager`] must be aware of that and correctly handle these.
///
/// If the backend doesn't support a feature in this config, it is free to ignore it. Is
/// responsibility of the user to know which of the features are available in the given backend.
#[derive(Debug, Copy, Clone, Default)]
pub struct TransactionConfig {
    /// The isolation level of the transaction.
    pub isolation: Option<IsolationLevel>,
    /// Is the transaction read-only?
    pub read_only: Option<bool>,
}

/// The isolation level of a database transaction.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum IsolationLevel {
    /// An individual statement in the transaction will see rows committed before it began.
    ReadCommitted,
    /// All statements in the transaction will see the same view of rows committed before the
    /// first query in the transaction.
    RepeatableRead,
    /// The read and writes in this transaction must be able to be committed as a single "unit"
    /// with respect to reads and writes of all other concurrent transactions without interleaving.
    Serializable,
}

/// Manages the transaction state of a [`RawConnection`].
///
/// Implementations of this trait are responsible to track the depth of the current transaction,
/// i.e. guarantee that nested transactions are supported _if_ the back supports it. They also
/// are responsible whether the connection is in a broken state or not, that is, if a rollback
/// failed, and the connection is inside an unaborted broken transaction.
///
/// Due to this, is recommended that implementations of this trait retry operations when they
/// fail to I/O errors.
pub trait TransactionManager<Conn>: Send + Sync
where
    Conn: RawConnection,
{
    /// Begin the transaction using the given configuration.
    ///
    /// Implementations can ignore the configuration when starting nested transactions
    /// if the backend doesn't support an individual configuration for them.
    fn begin_transaction<'c>(
        &'c self,
        config: TransactionConfig,
        conn: &'c Conn,
    ) -> BoxFuture<'c, QueryResult<()>>;

    /// Commit the transaction.
    ///
    /// Is expected that the implementation rollback the transaction if the `COMMIT` operation
    /// failed due to a serialization error.
    fn commit_transaction<'c>(&'c self, conn: &'c Conn) -> BoxFuture<'c, QueryResult<()>>;

    /// Rollbacks the transaction.
    fn rollback_transaction<'c>(&'c self, conn: &'c Conn) -> BoxFuture<'c, QueryResult<()>>;

    /// Returns whether the connection is in a broken state.
    fn is_broken(&self) -> bool;
}

/// A transaction manager that does nothing.
///
/// Useful when implementing backends that doesn't support transactions.
pub struct NoopTransactionManager;

impl<Conn> TransactionManager<Conn> for NoopTransactionManager
where
    Conn: RawConnection,
{
    fn begin_transaction<'c>(
        &'c self,
        _config: TransactionConfig,
        _conn: &'c Conn,
    ) -> BoxFuture<'c, QueryResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn commit_transaction<'c>(&'c self, _conn: &'c Conn) -> BoxFuture<'c, QueryResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_transaction<'c>(&'c self, _conn: &'c Conn) -> BoxFuture<'c, QueryResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn is_broken(&self) -> bool {
        false
    }
}

/// A future which executes the inner future inside a database transaction.
#[pin_project]
pub struct Transaction<'c, Conn, F>
where
    F: TryFuture,
{
    conn: &'c Conn,
    #[pin]
    state: TransactionState<'c, F>,
}

impl<'c, Conn, F> Transaction<'c, Conn, F>
where
    Conn: RawConnection,
    F: TryFuture,
{
    pub(super) fn new(conn: &'c Conn, inner: F) -> Self {
        Self {
            conn,
            state: TransactionState::NotStarted(Some(inner), Some(TransactionConfig::default())),
        }
    }

    /// Sets the isolation level of the transaction.
    pub fn isolation_level(mut self, level: IsolationLevel) -> Self {
        match &mut self.state {
            TransactionState::NotStarted(_, Some(conf)) => conf.isolation = Some(level),
            _ => unreachable!("Moved a started Transaction future!"),
        }
        self
    }

    /// Sets the access mode of the transaction.
    pub fn read_only(mut self) -> Self {
        match &mut self.state {
            TransactionState::NotStarted(_, Some(conf)) => conf.read_only = Some(true),
            _ => unreachable!("Moved a started Transaction future!"),
        }
        self
    }
}

/// Current state of [`Transaction`].
#[pin_project(project = StateProj)]
enum TransactionState<'c, F>
where
    F: TryFuture,
{
    /// The transaction is still not started.
    NotStarted(Option<F>, Option<TransactionConfig>),
    /// The transaction is starting.
    Beginning(#[pin] BoxFuture<'c, QueryResult<()>>, Option<F>),
    /// The transaction is in progress.
    InProgress(#[pin] CatchUnwind<AssertUnwindSafe<F>>),
    /// The transaction is committing.
    Committing {
        /// The commit future.
        #[pin]
        inner: BoxFuture<'c, QueryResult<()>>,
        /// The result of the transaction.
        output: Option<F::Ok>,
    },
    /// The transaction is aborting.
    Aborting {
        /// The commit future.
        #[pin]
        inner: BoxFuture<'c, QueryResult<()>>,
        /// The result of the transaction.
        output: Option<F::Error>,
    },
    /// The transaction panicked and is aborting.
    Panicking {
        /// Abort future.
        #[pin]
        inner: BoxFuture<'c, QueryResult<()>>,
        /// The panic payload.
        payload: Option<Box<dyn std::any::Any + Send>>,
    },
}

impl<Conn, F, T, E> Future for Transaction<'_, Conn, F>
where
    Conn: RawConnection,
    F: Future<Output = Result<T, E>>,
    E: From<Error>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use futures_util::future::FutureExt;

        let mut me = self.project();

        let next = match me.state.as_mut().project() {
            StateProj::NotStarted(inner, config) => {
                let tm = me.conn.transaction_manager();
                let begin = tm.begin_transaction(config.take().unwrap(), me.conn);
                TransactionState::Beginning(begin, inner.take())
            }
            StateProj::Beginning(begin, inner) => {
                if let Err(err) = ready!(begin.poll(cx)) {
                    return Poll::Ready(Err(err.into()));
                }

                TransactionState::InProgress(AssertUnwindSafe(inner.take().unwrap()).catch_unwind())
            }
            StateProj::InProgress(inner) => {
                match ready!(inner.try_poll(cx)) {
                    // The future didn't panic and resolved correctly, commit the transaction.
                    Ok(Ok(ok)) => {
                        let tm = me.conn.transaction_manager();
                        let inner = tm.commit_transaction(me.conn);
                        TransactionState::Committing {
                            inner,
                            output: Some(ok),
                        }
                    }
                    // The future didn't panic but resolved to an error, rollback the transaction.
                    Ok(Err(err)) => {
                        let tm = me.conn.transaction_manager();
                        let inner = tm.rollback_transaction(me.conn);
                        TransactionState::Aborting {
                            inner,
                            output: Some(err),
                        }
                    }
                    // The future panicked, rollback the transaction and resume unwind.
                    Err(payload) => {
                        let tm = me.conn.transaction_manager();
                        let inner = tm.rollback_transaction(me.conn);
                        TransactionState::Panicking {
                            inner,
                            payload: Some(payload),
                        }
                    }
                }
            }
            StateProj::Committing { inner, output } => {
                return match ready!(inner.poll(cx)) {
                    Ok(_) => Poll::Ready(Ok(output.take().unwrap())),
                    Err(err) => Poll::Ready(Err(err.into())),
                }
            }
            StateProj::Aborting { inner, output } => {
                return match ready!(inner.poll(cx)) {
                    Ok(_) => Poll::Ready(Err(output.take().unwrap())),
                    // Should we return the abort error here? I'm following the diesel
                    // behaviour but I'm not sure if this is the best one.
                    Err(err) => Poll::Ready(Err(err.into())),
                };
            }
            StateProj::Panicking { inner, payload } => {
                // TODO: What to do in case this fails?
                //   We're panicking, so we can just log and forget?
                let _ = ready!(inner.poll(cx));
                std::panic::resume_unwind(payload.take().unwrap())
            }
        };

        me.state.set(next);

        todo!()
    }
}
