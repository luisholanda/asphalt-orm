use crate::connection::{IsolationLevel, RawConnection, TransactionConfig, TransactionManager};
use crate::error::QueryResult;
use futures_core::future::LocalBoxFuture;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// An implementation of [`TransactionManager`] for the backends which use the ANSI syntax
/// for transactions and savepoint, such as PostgreSQL and SQLite.
#[derive(Debug, Default)]
pub struct AnsiTransactionManager {
    depth: AtomicU8,
    broken: AtomicBool,
}

impl AnsiTransactionManager {
    fn current_depth(&self) -> u8 {
        self.depth.load(Ordering::Acquire)
    }

    fn increment_depth(&self, query: QueryResult<()>) -> QueryResult<()> {
        if query.is_ok() {
            self.depth.fetch_add(1, Ordering::Relaxed);
        }
        query
    }

    fn decrement_depth(&self, query: QueryResult<()>) -> QueryResult<()> {
        if query.is_ok() {
            self.depth.fetch_sub(1, Ordering::Relaxed);
        }
        query
    }

    fn set_broken(&self) {
        self.broken.store(true, Ordering::Release);
    }

    fn first_transaction<Db>(&self, config: TransactionConfig) -> String {
        let mut stmt = String::from("BEGIN");

        if config.read_only == Some(true) {
            stmt.push_str(" READ ONLY");
        }

        if let Some(lvl) = config.isolation {
            stmt.push_str(" ISOLATION LEVEL ");

            stmt.push_str(match lvl {
                IsolationLevel::ReadCommitted => "READ COMMITTED",
                IsolationLevel::RepeatableRead => "REPEATABLE READ",
                IsolationLevel::Serializable => "SERIALIZABLE",
            });
        }

        stmt
    }
}

impl<Conn> TransactionManager<Conn> for AnsiTransactionManager
where
    Conn: RawConnection,
{
    fn begin_transaction<'c>(
        &'c self,
        config: TransactionConfig,
        conn: &'c Conn,
    ) -> LocalBoxFuture<'c, QueryResult<()>> {
        Box::pin(async move {
            let depth = self.current_depth();

            let stmt = if depth == 0 {
                self.first_transaction::<Conn::Backend>(config)
            } else {
                format!("SAVEPOINT asphalt_savepoint_{}", depth)
            };

            let res = conn.simple_execute(&stmt).await;
            self.increment_depth(res)
        })
    }

    fn commit_transaction<'c>(&'c self, conn: &'c Conn) -> LocalBoxFuture<'c, QueryResult<()>> {
        Box::pin(async move {
            let depth = self.current_depth();
            match depth {
                0 => panic!("Tried to commit with no transaction opened!"),
                1 => match conn.simple_execute("COMMIT").await {
                    Err(err) => {
                        if err.kind().is_serialization_failure()
                            || err.kind().is_read_only_transaction()
                        {
                            if let Err(err) =
                                self.decrement_depth(conn.simple_execute("ROLLBACK").await)
                            {
                                self.set_broken();
                                return Err(err);
                            }
                        }
                        Err(err)
                    }
                    e => self.decrement_depth(e),
                },
                _ => {
                    let qry = format!("RELEASE SAVEPOINT asphalt_savepoint_{}", depth - 1);
                    let res = conn.simple_execute(&qry).await;

                    self.decrement_depth(res)
                }
            }
        })
    }

    fn rollback_transaction<'c>(&'c self, conn: &'c Conn) -> LocalBoxFuture<'c, QueryResult<()>> {
        Box::pin(async move {
            let depth = self.current_depth();
            match depth {
                0 => panic!("Tried to rollback with no transaction opened!"),
                1 => self.decrement_depth(conn.simple_execute("ROLLBACK").await),
                _ => {
                    let qry = format!("ROLLBACK TO SAVEPOINT asphalt_savepoint_{}", depth - 1);
                    let res = conn.simple_execute(&qry).await;

                    self.decrement_depth(res)
                }
            }
        })
    }

    fn is_broken(&self) -> bool {
        self.broken.load(Ordering::Relaxed)
    }
}
