pub mod macros_utils;

use crate::Result;
use crate::{error::Error, ModelManager};
use modql::{
    field::HasSeaFields,
    filter::{FilterGroups, ListOptions},
};
use sea_query::{Condition, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, prelude::FromRow};
use uuid::Uuid;

use super::{PaginationMetadata, DMC, LIST_LIMIT_DEFAULT, LIST_LIMIT_MAX};

// DMC -> Database Model Control
pub async fn create<MC, I, O>(db: &ModelManager, input: I) -> Result<O>
where
    MC: DMC,
    I: HasSeaFields,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Setup Data
    let fields = input.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    // Preparing Query
    let mut query = Query::insert();
    query.into_table(MC::table_ref()).columns(columns).values(sea_values)?;

    // Returning
    let o_fields = O::sea_column_refs();
    query.returning(Query::returning().columns(o_fields));

    // Execute
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);

    match db.dbx().fetch_one(sqlx_query).await {
        Ok(entity) => Ok(entity),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn create_many<MC, I, O>(db: &ModelManager, input: Vec<I>) -> Result<Vec<O>>
where
    MC: DMC,
    I: HasSeaFields,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Setup Data
    let mut entities: Vec<O> = Vec::with_capacity(input.len());

    // Preparing Query
    let mut query = Query::insert();
    for item in input {
        let fields = item.not_none_sea_fields();
        let (columns, sea_values) = fields.for_sea_insert();
        query.into_table(MC::table_ref()).columns(columns).values(sea_values)?;
    }

    // Returning
    let o_fields = O::sea_column_refs();
    query.returning(Query::returning().columns(o_fields));

    // Execute
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
    let rows = db.dbx().fetch_all(sqlx_query).await?;

    for entity in rows {
        entities.push(entity);
    }

    Ok(entities)
}

pub async fn get_by_id<MC, O>(db: &ModelManager, id: Uuid) -> Result<O>
where
    MC: DMC,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(O::sea_column_refs())
        .and_where(Expr::col(MC::ID).eq(id));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
    let entity = db
        .dbx()
        .fetch_optional(sqlx_query)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id: 0,
        })?;

    Ok(entity)
}

pub async fn first<MC, F, O>(
    db: &ModelManager,
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
            // Reset the offset/limit
            list_options.offset = None;
            list_options.limit = Some(1);

            // Don't change order_bys if not empty,
            // otherwise, set it to id (creation asc order)
            list_options.order_bys = list_options.order_bys.or_else(|| Some(MC::ID.to_string().into()));

            list_options
        }
        None => ListOptions {
            limit: Some(1),
            offset: None,
            order_bys: Some(MC::ID.to_string().into()), // default id asc
        },
    };

    list::<MC, F, O>(db, filter, Some(list_options))
        .await
        .map(|(item, _)| item.into_iter().next())
}

pub async fn get_by_sth<MC, F, O>(db: &ModelManager, filter: Option<F>) -> Result<O>
where
    MC: DMC,
    F: Into<FilterGroups>,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // -- Build the query

    // let filter = filter.ok_or_else(|| AppError::BadRequest("Filter is required".to_string()))?;
    let mut query = Query::select()
        .from(MC::table_ref())
        .columns(O::sea_column_refs())
        .to_owned();

    // condition from filter
    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values.clone());
    let entity = db
        .dbx()
        .fetch_optional(sqlx_query)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id: 0000,
        })?;

    Ok(entity)
}

pub async fn list<MC, F, O>(
    db: &ModelManager,
    filter: Option<F>,
    list_options: Option<ListOptions>,
) -> Result<(Vec<O>, PaginationMetadata)>
where
    MC: DMC,
    F: Into<FilterGroups>,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    let (list_options, page) = compute_list_options::<MC>(list_options)?;

    let mut query = Query::select();
    query.from(MC::table_ref()).columns(O::sea_column_refs());

    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond.clone());
    }

    // Apply pagination to the main query
    let per_page = list_options.limit.unwrap_or(LIST_LIMIT_DEFAULT) as u64;
    list_options.apply_to_sea_query(&mut query);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
    let entities = db.dbx().fetch_all(sqlx_query).await?;

    let total_items = entities.len() as u64;
    let total_pages = (total_items as f64 / per_page as f64).ceil() as u64;

    let metadata = PaginationMetadata {
        current_page: page,
        per_page,
        total_items,
        total_pages,
    };

    Ok((entities, metadata))
}

pub async fn count<MC, F>(db: &ModelManager, filter: Option<F>) -> Result<i64>
where
    MC: DMC,
    F: Into<FilterGroups>,
{
    let db = db.dbx().db();
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

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let count: i64 = sqlx::query_scalar_with(&sql, values)
        .fetch_one(db)
        .await
        .map_err(|_| Error::CountFail)?;

    Ok(count)
}

pub async fn update<MC, I>(db: &ModelManager, id: i64, input: I) -> Result<()>
where
    MC: DMC,
    I: HasSeaFields,
{
    let fields = input.not_none_sea_fields();
    let fields = fields.for_sea_update();

    let mut query = Query::update();
    query
        .table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(MC::ID).eq(id));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let sqlx_query = sqlx::query_with(&sql, values);
    let count = db.dbx().execute(sqlx_query).await?;
    if count == 0 {
        Err(Error::EntityNotFound { entity: MC::TABLE, id })?
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(db: &ModelManager, id: i64) -> Result<()>
where
    MC: DMC,
{
    let mut query = Query::delete();
    query.from_table(MC::table_ref()).and_where(Expr::col(MC::ID).eq(id));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_with(&sql, values);
    let count = db.dbx().execute(sqlx_query).await?;

    if count == 0 {
        Err(Error::EntityNotFound { entity: MC::TABLE, id })?
    } else {
        Ok(())
    }
}

pub async fn delete_many<MC: DMC>(db: &ModelManager, ids: Vec<i64>) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }

    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(MC::ID).is_in(ids.clone()));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_with(&sql, values);
    let result = db.dbx().execute(sqlx_query).await?;

    if result as usize != ids.len() {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id: 0,
        })
    } else {
        Ok(())
    }
}

pub fn compute_list_options<MC: DMC>(list_options: Option<ListOptions>) -> Result<(ListOptions, u64)> {
    let mut list_options = list_options.unwrap_or_default();

    // Set default limit if not provided
    let limit = list_options.limit.unwrap_or(LIST_LIMIT_DEFAULT).min(LIST_LIMIT_MAX);
    list_options.limit = Some(limit);

    // Calculate current page based on offset and limit
    let offset = list_options.offset.unwrap_or(0).max(0);
    let page = (offset / limit) + 1;

    Ok((list_options, page as u64))
}
