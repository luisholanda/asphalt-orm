use crate::Pg;
use asphalt_core::backend::{Backend, HasSqlType, TypeMetadata};
use asphalt_core::error::{AnyResult, Error, QueryResult};
use asphalt_core::query::{BindCollector, PreparableQuery, QueryWriter};
use asphalt_core::types::ToSql;
use asphalt_core::LocalBoxFuture;
use bytes::{Bytes, BytesMut};
use tokio_postgres::types::{IsNull, Type};
use tokio_postgres::Statement;

pub struct PgQuery {
    inner: InnerQuery,
}

enum InnerQuery {
    Raw(String, Vec<Type>),
    Stmt(Statement),
}

impl PreparableQuery<Pg> for PgQuery {
    type Prepared = Statement;

    fn prepare(
        self,
        conn: &<Pg as Backend>::RawConnection,
    ) -> LocalBoxFuture<'_, QueryResult<Self::Prepared>> {
        match self.inner {
            InnerQuery::Stmt(stmt) => Box::pin(async move { Ok(stmt) }),
            InnerQuery::Raw(raw, types) => Box::pin(async move {
                conn.inner
                    .prepare_typed(&raw, &types)
                    .await
                    .map_err(crate::error_to_query_error)
            }),
        }
    }

    fn from_prepared(prepared: Self::Prepared) -> Self {
        Self {
            inner: InnerQuery::Stmt(prepared),
        }
    }
}

/// The `QueryWriter` for the `Pg` backend.
#[derive(Default)]
pub struct PgQueryWriter {
    query: String,
    types: Vec<Type>,
    has_unknown: bool,
}

impl QueryWriter<Pg> for PgQueryWriter {
    fn push_sql(&mut self, sql: &str) {
        self.query.push_str(sql);
    }

    fn push_identifier(&mut self, identifier: &str) {
        self.query.reserve(2 + identifier.len());
        self.query.push('"');
        self.query.push_str(&identifier.replace('"', "\"\""));
        self.query.push('"');
    }

    fn push_bind_param(&mut self, bind: &<Pg as Backend>::BindName) {
        use std::fmt::Write;
        // Writing to memory never fails.
        write!(&mut self.query, "${}", bind.0).unwrap();
        // Only worry about parameter types metadata if we know all of them till now.
        if !self.has_unknown {
            if let Some(typ) = &bind.1 {
                self.types.push(typ.clone());
            } else {
                // We don't know this parameter type metadata.
                // Ignore all the remaining ones.
                self.has_unknown = true;
            }
        }
    }

    fn finish(self) -> <Pg as Backend>::Query {
        PgQuery {
            inner: InnerQuery::Raw(self.query, self.types),
        }
    }
}

#[derive(Default)]
pub struct PgBindCollector {
    binds: Vec<PgParam>,
    buffer: BytesMut,
}

impl PgBindCollector {
    pub(crate) fn buffer(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    pub(crate) fn binds(
        &self,
    ) -> impl Iterator<Item = &dyn tokio_postgres::types::ToSql> + std::iter::ExactSizeIterator
    {
        self.binds
            .iter()
            .map(|p| p as &dyn tokio_postgres::types::ToSql)
    }
}

impl BindCollector<Pg> for PgBindCollector {
    fn push_bound_value<'a, SqlTy, RustTy>(
        &'a mut self,
        bind: &'a RustTy,
        metadata_lookup: &'a <Pg as TypeMetadata>::MetadataLookup,
    ) -> LocalBoxFuture<'a, QueryResult<<Pg as Backend>::BindName>>
    where
        Pg: HasSqlType<SqlTy>,
        RustTy: ToSql<SqlTy, Pg>,
    {
        Box::pin(async move {
            let metadata = <Pg as HasSqlType<SqlTy>>::metadata(metadata_lookup).await?;
            bind.to_sql(&metadata, self)
                .map_err(Error::serialization_failure)?;

            self.binds.push(PgParam(self.buffer.split().freeze()));

            // TODO: error if too many parameters
            Ok((self.binds.len() as u16, metadata))
        })
    }
}

#[derive(Debug)]
pub(crate) struct PgParam(pub(crate) Bytes);

impl tokio_postgres::types::ToSql for PgParam {
    fn to_sql(&self, _ty: &Type, out: &mut BytesMut) -> AnyResult<IsNull>
    where
        Self: Sized,
    {
        if self.0.is_empty() {
            Ok(IsNull::Yes)
        } else {
            out.extend_from_slice(&self.0);
            Ok(IsNull::No)
        }
    }

    fn accepts(_ty: &Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    tokio_postgres::types::to_sql_checked!();
}
