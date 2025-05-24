use crate::Result;
mod utils;
use modql::{field::HasSeaFields, SIden};
use sea_query::{Iden, PostgresQueryBuilder, Query, SeaRc, TableRef};
use sea_query_binder::SqlxBinder;
use serde::Serialize;
use sqlx::{postgres::PgRow, prelude::FromRow};
use utils::prepare_fields_for_create;

use crate::{ctx::Ctx, ModelManager};

// -->>> Region:: START  --->>>  Constants
const LIST_LIMIT_DEFAULT: i64 = 20;
const LIST_LIMIT_MAX: i64 = 50;
// <<<-- Region:: END    <<<---  Constants

#[derive(Iden)]
pub enum CommonId {
    UserId,
    OwnerId,
    BlogId,
    CourseId,
}

#[derive(Iden)]
pub enum TimestampIden {
    Cid,
    Ctime,
    Mid,
    Mtime,
}

#[derive(Serialize)]
pub struct PaginationMetadata {
    current_page: u64,
    per_page: u64,
    total_item: u64,
    total_pages: u64,
}

pub trait DMC {
    const SCHEMA: &'static str;
    const TABLE: &'static str;

    fn table_ref() -> TableRef {
        TableRef::SchemaTable(SeaRc::new(SIden(Self::SCHEMA)), SeaRc::new(SIden(Self::TABLE)))
    }

    /// Specifies that the table for this Bmc has timestamps (cid, ctime, mid, mtime) columns.
    /// This will allow the code to update those as needed.
    ///
    /// default: true
    fn has_timestamps() -> bool {
        true
    }

    /// Specifies if the entity table managed by this BMC
    /// has an `owner_id` column that needs to be set on create (by default ctx.user_id).
    ///
    /// default: false
    fn has_owner_id() -> bool {
        false
    }
}

pub async fn create<MC, I, O>(ctx: &Ctx, mm: &ModelManager, input: I) -> Result<O>
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
    query.into_table(MC::table_ref()).columns(columns).values(sea_values)?;

    // -- Build Returning
    let o_fields = O::sea_column_refs();
    query.returning(Query::returning().columns(o_fields));

    // Execute Query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);

    let entity = mm.dbx().fetch_one(sqlx_query).await?;

    Ok(entity)
}

pub async fn create_many<MC, I, O>(ctx: &Ctx, mm: &ModelManager, input: Vec<I>) -> Result<Vec<O>>
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
