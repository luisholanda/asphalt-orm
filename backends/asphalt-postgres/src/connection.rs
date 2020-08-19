use crate::metadata::MetadataLookup;
use crate::Pg;
use asphalt_core::backend::{HasSqlType, TypeMetadata};
use asphalt_core::connection::{EstablishResult, RawConnection, Row, RowStream};
use asphalt_core::error::{AnyResult, QueryResult};
use asphalt_core::query::{PreparableQuery, Query};
use asphalt_core::sql::AnsiTransactionManager;
use asphalt_core::types::FromSql;
use asphalt_core::LocalBoxFuture;
use tokio::stream::StreamExt;
use tokio_postgres::{types::Type, Client, Config as PgConfig, NoTls};
#[cfg(feature = "tls")]
use tokio_postgres_rustls::MakeRustlsConnect;

#[doc(inline)]
pub type ConnectionConfig = PgConfig;

pub struct Config {
    connection: ConnectionConfig,
    #[cfg(feature = "tls")]
    tls: rustls::ClientConfig,
}

impl Config {
    pub fn new(connection_config: ConnectionConfig) -> Self {
        Self {
            connection: connection_config,
            #[cfg(feature = "tls")]
            tls: rustls::ClientConfig::new(),
        }
    }

    #[cfg(feature = "tls")]
    pub fn set_tls_config(mut self, config: rustls::ClientConfig) -> Self {
        self.tls = config;
        self
    }
}

pub struct PgRawConnection {
    pub(crate) inner: Client,
    manager: AnsiTransactionManager,
    metadata: MetadataLookup,
}

impl PgRawConnection {
    #[cfg(feature = "tls")]
    async fn connect(config: Config) -> EstablishResult<Self> {
        let tls = MakeRustlsConnect::new(config.tls);

        let (client, connection) = config.connection.connect(tls).await?;
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                eprintln!("connection error: {}", err)
            }
        });

        Ok(Self {
            inner: client,
            manager: AnsiTransactionManager::default(),
            metadata: MetadataLookup::default(),
        })
    }

    #[cfg(not(feature = "tls"))]
    async fn connect(config: Config) -> EstablishResult<Self> {
        let (client, connection) = config.connection.connect(NoTls).await?;
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                eprintln!("connection error: {}", err)
            }
        });

        Ok(Self {
            inner: client,
            manager: AnsiTransactionManager::default(),
            metadata: MetadataLookup::default(),
        })
    }
}

impl RawConnection for PgRawConnection {
    type Backend = Pg;
    type TransactionManager = AnsiTransactionManager;
    type Row = PgRow;
    type Config = Config;
    type EstablishError = tokio_postgres::error::Error;

    fn establish(config: Self::Config) -> LocalBoxFuture<'static, EstablishResult<Self>> {
        Box::pin(Self::connect(config))
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        &self.manager
    }

    fn simple_execute<'s>(&'s self, sql: &'s str) -> LocalBoxFuture<'s, QueryResult<()>> {
        Box::pin(async move {
            Ok(self
                .inner
                .batch_execute(sql)
                .await
                .map_err(crate::error_to_query_error)?)
        })
    }

    fn execute(&self, query: Query<Self::Backend>) -> LocalBoxFuture<'_, QueryResult<u64>> {
        Box::pin(async move {
            let stmt = query.inner.prepare(self).await?;

            self.inner
                .execute_raw(&stmt, query.binds.binds())
                .await
                .map_err(crate::error_to_query_error)
        })
    }

    fn query(
        &self,
        query: Query<Self::Backend>,
    ) -> LocalBoxFuture<'_, QueryResult<RowStream<'_, Self>>> {
        Box::pin(async move {
            let stmt = query.inner.prepare(self).await?;

            let stream = self
                .inner
                .query_raw(&stmt, query.binds.binds())
                .await
                .map_err(crate::error_to_query_error)?;

            let mut stream = Box::pin(stream).map(|r| r.map_err(crate::error_to_query_error));
            if let Some(row) = stream.try_next().await? {
                // Register the columns types metadata so we can use them in parameters.
                for col in row.columns() {
                    self.metadata.register_type_metadata(col.type_().clone());
                }

                let stream = tokio::stream::once(Ok(row)).chain(stream);

                Ok(Box::pin(stream.map(|r| r.map(|inner| PgRow { inner }))) as RowStream<'_, Self>)
            } else {
                // The stream is empty.
                Ok(Box::pin(tokio::stream::empty()) as RowStream<'_, Self>)
            }
        })
    }

    fn metadata_lookup(&self) -> &<Self::Backend as TypeMetadata>::MetadataLookup {
        &self.metadata
    }
}

pub struct PgRow {
    inner: tokio_postgres::Row,
}

impl Row for PgRow {
    type Backend = Pg;

    fn n_columns(&self) -> usize {
        self.inner.len()
    }

    fn get_column<'a, SqlTy, RustTy>(&'a self, idx: usize) -> AnyResult<RustTy>
    where
        Self::Backend: HasSqlType<SqlTy>,
        RustTy: FromSql<'a, SqlTy, Self::Backend>,
    {
        let col = self.inner.try_get::<_, PgRowCol>(idx)?.0;
        let metadata = self.inner.columns()[idx].type_().clone();
        RustTy::from_sql(&Some(metadata), col)
    }
}

struct PgRowCol<'b>(&'b [u8]);

impl<'a> tokio_postgres::types::FromSql<'a> for PgRowCol<'a> {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> AnyResult<Self> {
        Ok(Self(raw))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
