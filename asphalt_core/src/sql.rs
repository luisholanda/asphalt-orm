use crate::connection::{IsolationLevel, RawConnection, TransactionConfig, TransactionManager};
use crate::error::QueryResult;
use crate::extensions::{supports, ReadOnly, Supports, Transaction};
use futures_core::future::BoxFuture;
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

        if supports::<Db, ReadOnly>() && config.read_only == Some(true) {
            stmt.push_str(" READ ONLY");
        }

        if supports::<Db, IsolationLevel>() {
            if let Some(lvl) = config.isolation {
                stmt.push_str(" ISOLATION LEVEL ");

                stmt.push_str(match lvl {
                    IsolationLevel::ReadCommitted => "READ COMMITTED",
                    IsolationLevel::RepeatableRead => "REPEATABLE READ",
                    IsolationLevel::Serializable => "SERIALIZABLE",
                });
            }
        }

        stmt
    }
}

impl<Conn> TransactionManager<Conn> for AnsiTransactionManager
where
    Conn: RawConnection + Supports<Transaction>,
{
    fn begin_transaction<'c>(
        &'c self,
        config: TransactionConfig,
        conn: &'c Conn,
    ) -> BoxFuture<'c, QueryResult<()>> {
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

    fn commit_transaction<'c>(&'c self, conn: &'c Conn) -> BoxFuture<'c, QueryResult<()>> {
        Box::pin(async move {
            let depth = self.current_depth();
            match depth {
                0 => panic!("Tried to commit with no transaction opened!"),
                1 => match conn.simple_execute("COMMIT").await {
                    e => self.decrement_depth(),
                },
            }
        })
    }

    fn rollback_transaction<'c>(&'c self, conn: &'c Conn) -> BoxFuture<'c, QueryResult<()>> {
        unimplemented!()
    }

    fn is_broken(&self) -> bool {
        unimplemented!()
    }
}
