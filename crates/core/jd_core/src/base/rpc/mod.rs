pub mod handlers;
pub mod macros_utils;
pub mod prelude;
mod rpc_params;
mod rpc_result;
mod utils;

pub use rpc_params::*;
pub use rpc_result::*;

use crate::{Result, error::Error};
use modql::{
  field::HasSeaFields,
  filter::{FilterGroups, ListOptions},
};

use sea_query::{Condition, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{Row, postgres::PgRow, prelude::FromRow};
use utils::{prepare_fields_for_create, prepare_fields_for_update};

use crate::{ModelManager, ctx::Ctx};

use super::{CommonId, DMC, LIST_LIMIT_DEFAULT, LIST_LIMIT_MAX};

pub async fn ctx_create<MC, I, O>(ctx: &Ctx, mm: &ModelManager, input: I) -> Result<O>
where
  MC: DMC,
  I: HasSeaFields,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  let user_id = ctx.user_id();

  // -- Extract fields name
  let mut fields = input.not_none_sea_fields();
  prepare_fields_for_create::<MC>(&mut fields, user_id);

  // -- Build Query
  let (columns, sea_values) = fields.for_sea_insert();
  let mut query = Query::insert();
  query
    .into_table(MC::table_ref())
    .columns(columns)
    .values(sea_values)?;

  // -- Build Returning
  let o_fields = O::sea_column_refs();
  query.returning(Query::returning().columns(o_fields));

  // Execute Query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);

  let entity = mm.dbx().fetch_one(sqlx_query).await?;

  Ok(entity)
}

pub async fn ctx_create_many<MC, I, O>(
  ctx: &Ctx,
  mm: &ModelManager,
  input: Vec<I>,
) -> Result<Vec<O>>
where
  MC: DMC,
  I: HasSeaFields,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  let user_id = ctx.user_id();
  let mut entities: Vec<O> = Vec::with_capacity(input.len());

  let mut query = Query::insert();

  for item in input {
    let mut fields = item.not_none_sea_fields();
    prepare_fields_for_create::<MC>(&mut fields, user_id);
    let (columns, sea_values) = fields.for_sea_insert();

    query
      .into_table(MC::table_ref())
      .columns(columns.clone())
      .values(sea_values)?;
  }

  let o_fields = O::sea_column_refs();
  query.returning(Query::returning().columns(o_fields));

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);

  let rows = mm.dbx().fetch_all(sqlx_query).await?;

  for entity in rows {
    entities.push(entity);
  }

  Ok(entities)
}

pub async fn ctx_get<MC, O>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<O>
where
  MC: DMC,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  let mut query = Query::select();
  query
    .from(MC::table_ref())
    .columns(O::sea_column_refs())
    .and_where(Expr::col(CommonId::Id).eq(id));

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
  let entity = mm
    .dbx()
    .fetch_optional(sqlx_query)
    .await?
    .ok_or(Error::EntityNotFound { entity: MC::TABLE, id })?;

  Ok(entity)
}

pub async fn ctx_first<MC, O, F>(
  ctx: &Ctx,
  mm: &ModelManager,
  filter: Option<F>,
  list_options: Option<ListOptions>,
) -> Result<Option<O>>
where
  MC: DMC,
  F: Into<FilterGroups>,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  let list_options = match list_options {
    Some(mut list_options) => {
      list_options.offset = None;
      list_options.limit = Some(1);
      list_options.order_bys = list_options.order_bys.or_else(|| Some("id".into()));
      list_options
    }
    None => ListOptions { limit: Some(1), offset: None, order_bys: Some("id".into()) },
  };
  ctx_list::<MC, O, F>(ctx, mm, filter, Some(list_options))
    .await
    .map(|item| item.into_iter().next())
}

pub async fn ctx_list<MC, O, F>(
  _ctx: &Ctx,
  mm: &ModelManager,
  filter: Option<F>,
  list_options: Option<ListOptions>,
) -> Result<Vec<O>>
where
  MC: DMC,
  F: Into<FilterGroups>,
  O: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  O: HasSeaFields,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(O::sea_column_refs());

  // condition from filter
  if let Some(filter) = filter {
    let filters: FilterGroups = filter.into();
    let cond: Condition = filters.try_into()?;
    query.cond_where(cond);
  }

  // list options
  let list_options = compute_list_options(list_options)?;
  list_options.apply_to_sea_query(&mut query);

  // -- Execute the query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
  let entities = mm.dbx().fetch_all(sqlx_query).await?;

  Ok(entities)
}

pub async fn ctx_count<MC, F>(_ctx: &Ctx, mm: &ModelManager, filter: Option<F>) -> Result<i64>
where
  MC: DMC,
  F: Into<FilterGroups>,
{
  let db = mm.dbx().db();
  // -- Build the query
  let mut query = Query::select()
    .from(MC::table_ref())
    .expr(Expr::col(sea_query::Asterisk).count())
    .to_owned();

  // condition from filter
  if let Some(filter) = filter {
    let filters: FilterGroups = filter.into();
    let cond: Condition = filters.try_into()?;
    query.cond_where(cond);
  }

  let query_str = query.to_string(PostgresQueryBuilder);

  let result = sqlx::query(&query_str)
    .fetch_one(db)
    .await
    .map_err(|_| Error::CountFail)?;

  let count: i64 = result.try_get("count").map_err(|_| Error::CountFail)?;

  Ok(count)
}

pub async fn ctx_update<MC, O>(ctx: &Ctx, mm: &ModelManager, id: i64, data: O) -> Result<()>
where
  MC: DMC,
  O: HasSeaFields,
{
  // -- Prep Fields
  let mut fields = data.not_none_sea_fields();
  prepare_fields_for_update::<MC>(&mut fields, ctx.user_id());

  // -- Build query
  let fields = fields.for_sea_update();
  let mut query = Query::update();
  query
    .table(MC::table_ref())
    .values(fields)
    .and_where(Expr::col(CommonId::Id).eq(id));

  // -- Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let count = mm.dbx().execute(sqlx_query).await?;

  // -- Check result
  if count == 0 { Err(Error::EntityNotFound { entity: MC::TABLE, id }) } else { Ok(()) }
}

pub async fn ctx_delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
  MC: DMC,
{
  // -- Build query
  let mut query = Query::delete();
  query
    .from_table(MC::table_ref())
    .and_where(Expr::col(CommonId::Id).eq(id));

  // -- Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let count = mm.dbx().execute(sqlx_query).await?;

  // -- Check result
  if count == 0 { Err(Error::EntityNotFound { entity: MC::TABLE, id }) } else { Ok(()) }
}

pub async fn ctx_delete_many<MC>(_ctx: &Ctx, mm: &ModelManager, ids: Vec<i64>) -> Result<u64>
where
  MC: DMC,
{
  if ids.is_empty() {
    return Ok(0);
  }

  // -- Build query
  let mut query = Query::delete();
  query
    .from_table(MC::table_ref())
    .and_where(Expr::col(CommonId::Id).is_in(ids.clone()));

  // -- Execute query
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let sqlx_query = sqlx::query_with(&sql, values);
  let result = mm.dbx().execute(sqlx_query).await?;

  // -- Check result
  if result as usize != ids.len() {
    Err(Error::EntityNotFound {
      entity: MC::TABLE,
      id: 0, // Using 0 because multiple IDs could be not found, you may want to improve error handling here
    })
  } else {
    Ok(result)
  }
}

pub fn compute_list_options(list_options: Option<ListOptions>) -> Result<ListOptions> {
  if let Some(mut list_options) = list_options {
    // Validate the limit.
    if let Some(limit) = list_options.limit {
      if limit > LIST_LIMIT_MAX {
        return Err(Error::ListLimitOverMax { max: LIST_LIMIT_MAX, actual: limit });
      }
    }
    // Set the default limit if no limit
    else {
      list_options.limit = Some(LIST_LIMIT_DEFAULT);
    }
    Ok(list_options)
  }
  // When None, return default
  else {
    Ok(ListOptions { limit: Some(LIST_LIMIT_DEFAULT), offset: None, order_bys: Some("id".into()) })
  }
}
