#![feature(
    associated_type_bounds,
    const_generics,
    generic_associated_types,
    marker_trait_attr
)]
mod access;
mod expressions;
mod schemas;

//
// let conn = pool.get().await?;
// let users = conn.from(users::table).filter(..).await?;
// let user = conn.from(users::table).get_one(id).await?;
// conn.update(users::table).set(..).filter(..).await?;
// conn.insert_into(users::table).values(..).await?;
// [user_id.eq(id): BoolOp<user_id, _, {Eq}>]
//
