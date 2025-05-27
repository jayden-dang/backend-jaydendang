pub mod macros_utils;

use crate::Result;
use crate::{error::Error, ModelManager};
use modql::{
    field::HasSeaFields,
    filter::{FilterGroups, ListOptions},
};
use sea_query::{Condition, Expr, PostgresQueryBuilder, Query, Value};
use sea_query_binder::{SqlxBinder, SqlxValues};
use sqlx::{postgres::PgRow, prelude::FromRow};
use uuid::Uuid;
use std::collections::HashMap;
use regex::Regex;

use super::{PaginationMetadata, DMC, LIST_LIMIT_DEFAULT, LIST_LIMIT_MAX};

#[derive(Debug, Clone)]
pub struct PgEnum {
    pub type_name: String,
    pub value: String,
}

/// Creates a single record in the database
///
/// # Arguments
/// * `db` - The database connection manager
/// * `input` - The data to create the record with
///
/// # Returns
/// * `Result<O>` - The created record or an error
///
/// # Example
/// ```rust
/// let input = CreateUserInput { name: "John".to_string() };
/// let user = create::<UserModel, _, User>(db, input).await?;
/// ```
pub async fn create<MC, I, O>(db: &ModelManager, input: I) -> Result<O>
where
    MC: DMC,
    I: HasSeaFields,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Step 1: Extract non-null fields from input and prepare for database insertion
    let fields = input.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    // Step 2: Build the INSERT query
    let mut query = Query::insert();
    query
        .into_table(MC::table_ref())
        .columns(columns)
        .values(sea_values)?;

    // Step 3: Add RETURNING clause to get the created record
    let o_fields = O::sea_column_refs();
    query.returning(Query::returning().columns(o_fields));

    // Step 4: Execute the query and handle the result
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    // 🔍 DEBUG: Log the generated SQL and values
    println!("Generated SQL: {}", sql);
    println!("Values: {:?}", values);

    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);

    match db.dbx().fetch_one(sqlx_query).await {
        Ok(entity) => Ok(entity),
        Err(e) => {
            // 🔍 DEBUG: Log the actual error
            println!("Database error: {:?}", e);

            match e {
                jd_storage::dbx::Error::Sqlx(sqlx_err) => {
                    // Handle unique constraint violation
                    if let Some(db_err) = sqlx_err.as_database_error() {
                        if db_err.code().map(|code| code == "23505").unwrap_or(false) {
                            return Err(Error::UniqueViolation {
                                table: db_err.table().unwrap_or("unknown").to_string(),
                                constraint: db_err.constraint().unwrap_or("unknown").to_string(),
                            });
                        }
                    }
                    Err(Error::Sqlx(sqlx_err))
                }
                _ => Err(Error::Dbx(e)),
            }
        }
    }
}
/// Creates multiple records in the database
///
/// # Arguments
/// * `db` - The database connection manager
/// * `input` - Vector of data to create records with
///
/// # Returns
/// * `Result<Vec<O>>` - Vector of created records or an error
///
/// # Example
/// ```rust
/// let inputs = vec![
///     CreateUserInput { name: "John".to_string() },
///     CreateUserInput { name: "Jane".to_string() }
/// ];
/// let users = create_many::<UserModel, _, User>(db, inputs).await?;
/// ```
pub async fn create_many<MC, I, O>(db: &ModelManager, input: Vec<I>) -> Result<Vec<O>>
where
    MC: DMC,
    I: HasSeaFields,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Step 1: Initialize result vector with capacity matching input size
    let mut entities: Vec<O> = Vec::with_capacity(input.len());

    // Step 2: Build the INSERT query for multiple records
    let mut query = Query::insert();
    for item in input {
        // Extract fields and prepare values for each record
        let fields = item.not_none_sea_fields();
        let (columns, sea_values) = fields.for_sea_insert();
        query
            .into_table(MC::table_ref())
            .columns(columns)
            .values(sea_values)?;
    }

    // Step 3: Add RETURNING clause to get all created records
    let o_fields = O::sea_column_refs();
    query.returning(Query::returning().columns(o_fields));

    // Step 4: Execute the query and collect results
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
    let rows = db.dbx().fetch_all(sqlx_query).await?;

    // Step 5: Convert rows to entities
    for entity in rows {
        entities.push(entity);
    }

    Ok(entities)
}

/// Retrieves a single record by its ID
///
/// # Arguments
/// * `db` - The database connection manager
/// * `id` - The ID of the record to retrieve
///
/// # Returns
/// * `Result<O>` - The found record or an error if not found
///
/// # Example
/// ```rust
/// let user = get_by_id::<UserModel, User>(db, user_id).await?;
/// ```
pub async fn get_by_id<MC, O>(db: &ModelManager, id: Uuid) -> Result<O>
where
    MC: DMC,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Step 1: Build SELECT query with ID condition
    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(O::sea_column_refs())
        .and_where(Expr::col(MC::ID).eq(id));

    // Step 2: Execute query and handle result
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
    let entity = db
        .dbx()
        .fetch_optional(sqlx_query)
        .await?
        .ok_or(Error::EntityNotFound { entity: MC::TABLE, id: 0 })?;

    Ok(entity)
}

/// Retrieves the first record matching the given filter
///
/// # Arguments
/// * `db` - The database connection manager
/// * `filter` - Optional filter conditions
/// * `list_options` - Optional list options for ordering and limiting
///
/// # Returns
/// * `Result<Option<O>>` - The first matching record or None if no matches
///
/// # Example
/// ```rust
/// let filter = UserFilter { status: Some("active".to_string()) };
/// let user = first::<UserModel, _, User>(db, Some(filter), None).await?;
/// ```
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
    // Step 1: Prepare list options for single record retrieval
    let list_options = match list_options {
        Some(mut list_options) => {
            // Reset pagination settings
            list_options.offset = None;
            list_options.limit = Some(1);

            // Set default ordering if not provided
            list_options.order_bys = list_options
                .order_bys
                .or_else(|| Some(MC::ID.to_string().into()));

            list_options
        }
        None => ListOptions {
            limit: Some(1),
            offset: None,
            order_bys: Some(MC::ID.to_string().into()), // default id asc
        },
    };

    // Step 2: Use list function to get first record
    list::<MC, F, O>(db, filter, Some(list_options))
        .await
        .map(|(item, _)| item.into_iter().next())
}

/// Retrieves a single record matching the given filter
///
/// # Arguments
/// * `db` - The database connection manager
/// * `filter` - Filter conditions to match the record
///
/// # Returns
/// * `Result<O>` - The matching record or an error if not found
///
/// # Example
/// ```rust
/// let filter = UserFilter { email: Some("user@example.com".to_string()) };
/// let user = get_by_sth::<UserModel, _, User>(db, Some(filter)).await?;
/// ```
pub async fn get_by_sth<MC, F, O>(db: &ModelManager, filter: Option<F>) -> Result<O>
where
    MC: DMC,
    F: Into<FilterGroups>,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Step 1: Build base SELECT query
    let mut query = Query::select()
        .from(MC::table_ref())
        .columns(O::sea_column_refs())
        .to_owned();

    // Step 2: Apply filter conditions if provided
    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond);
    }

    // Step 3: Execute query and handle result
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values.clone());
    let entity = db
        .dbx()
        .fetch_optional(sqlx_query)
        .await?
        .ok_or(Error::EntityNotFound { entity: MC::TABLE, id: 0000 })?;

    Ok(entity)
}

/// Lists records matching the given filter with pagination
///
/// # Arguments
/// * `db` - The database connection manager
/// * `filter` - Optional filter conditions
/// * `list_options` - Optional list options for pagination and ordering
///
/// # Returns
/// * `Result<(Vec<O>, PaginationMetadata)>` - Tuple of matching records and pagination metadata
///
/// # Example
/// ```rust
/// let filter = UserFilter { status: Some("active".to_string()) };
/// let list_options = ListOptions { limit: Some(10), offset: Some(0), ..Default::default() };
/// let (users, metadata) = list::<UserModel, _, User>(db, Some(filter), Some(list_options)).await?;
/// ```
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
    // Step 1: Compute list options and current page
    let (list_options, page) = compute_list_options::<MC>(list_options)?;

    // Step 2: Build base SELECT query
    let mut query = Query::select();
    query.from(MC::table_ref()).columns(O::sea_column_refs());

    // Step 3: Apply filter conditions if provided
    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond.clone());
    }

    // Step 4: Apply pagination settings
    let per_page = list_options.limit.unwrap_or(LIST_LIMIT_DEFAULT) as u64;
    list_options.apply_to_sea_query(&mut query);

    // Step 5: Execute query and get results
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
    let entities = db.dbx().fetch_all(sqlx_query).await?;

    // Step 6: Calculate pagination metadata
    let total_items = entities.len() as u64;
    let total_pages = (total_items as f64 / per_page as f64).ceil() as u64;

    let metadata = PaginationMetadata { current_page: page, per_page, total_items, total_pages };

    Ok((entities, metadata))
}

/// Counts records matching the given filter
///
/// # Arguments
/// * `db` - The database connection manager
/// * `filter` - Optional filter conditions
///
/// # Returns
/// * `Result<i64>` - The count of matching records
///
/// # Example
/// ```rust
/// let filter = UserFilter { status: Some("active".to_string()) };
/// let count = count::<UserModel, _>(db, Some(filter)).await?;
/// ```
pub async fn count<MC, F>(db: &ModelManager, filter: Option<F>) -> Result<i64>
where
    MC: DMC,
    F: Into<FilterGroups>,
{
    // Step 1: Get database connection
    let db = db.dbx().db();

    // Step 2: Build COUNT query
    let mut query = Query::select()
        .from(MC::table_ref())
        .expr(Expr::col(sea_query::Asterisk).count())
        .to_owned();

    // Step 3: Apply filter conditions if provided
    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond);
    }

    // Step 4: Execute query and get count
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count: i64 = sqlx::query_scalar_with(&sql, values)
        .fetch_one(db)
        .await
        .map_err(|_| Error::CountFail)?;

    Ok(count)
}

/// Updates a single record by its ID
///
/// # Arguments
/// * `db` - The database connection manager
/// * `id` - The ID of the record to update
/// * `input` - The data to update the record with
///
/// # Returns
/// * `Result<()>` - Success if the record was updated, Error if not found
///
/// # Example
/// ```rust
/// let input = UpdateUserInput { status: "inactive".to_string() };
/// update::<UserModel, _>(db, user_id, input).await?;
/// ```
pub async fn update<MC, I>(db: &ModelManager, id: Uuid, input: I) -> Result<()>
where
    MC: DMC,
    I: HasSeaFields,
{
    // Step 1: Extract non-null fields and prepare for update
    let fields = input.not_none_sea_fields();
    let fields = fields.for_sea_update();

    // Step 2: Build UPDATE query with ID condition
    let mut query = Query::update();
    query
        .table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(MC::ID).eq(id));

    // Step 3: Execute query and check if any record was updated
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_with(&sql, values);
    let result = db.dbx().execute(sqlx_query).await?;

    if result == 0 {
        Err(Error::EntityNotFound { entity: MC::TABLE, id: 0 })
    } else {
        Ok(())
    }
}

/// Deletes a single record by its ID
///
/// # Arguments
/// * `db` - The database connection manager
/// * `id` - The ID of the record to delete
///
/// # Returns
/// * `Result<()>` - Success if the record was deleted, Error if not found
///
/// # Example
/// ```rust
/// delete::<UserModel>(db, user_id).await?;
/// ```
pub async fn delete<MC>(db: &ModelManager, id: Uuid) -> Result<()>
where
    MC: DMC,
{
    // Step 1: Build DELETE query with ID condition
    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(MC::ID).eq(id));

    // Step 2: Execute query and check if any record was deleted
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_with(&sql, values);
    let result = db.dbx().execute(sqlx_query).await?;

    if result == 0 {
        Err(Error::EntityNotFound { entity: MC::TABLE, id: 0 })
    } else {
        Ok(())
    }
}

/// Deletes multiple records by their IDs
///
/// # Arguments
/// * `db` - The database connection manager
/// * `ids` - Vector of record IDs to delete
///
/// # Returns
/// * `Result<()>` - Success if all records were deleted, Error if none were found
///
/// # Example
/// ```rust
/// let ids = vec![user_id1, user_id2];
/// delete_many::<UserModel>(db, ids).await?;
/// ```
pub async fn delete_many<MC: DMC>(db: &ModelManager, ids: Vec<Uuid>) -> Result<()> {
    // Step 1: Early return if no IDs provided
    if ids.is_empty() {
        return Ok(());
    }

    // Step 2: Build DELETE query with multiple IDs
    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(MC::ID).is_in(ids.clone()));

    // Step 3: Execute query and check if any records were deleted
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_with(&sql, values);
    let result = db.dbx().execute(sqlx_query).await?;

    if result == 0 {
        Err(Error::EntityNotFound { entity: MC::TABLE, id: 0 })
    } else {
        Ok(())
    }
}

/// Computes list options for pagination
///
/// # Arguments
/// * `list_options` - Optional list options to compute
///
/// # Returns
/// * `Result<(ListOptions, u64)>` - Tuple of computed list options and current page number
///
/// # Example
/// ```rust
/// let list_options = ListOptions { limit: Some(10), offset: Some(20), ..Default::default() };
/// let (computed_options, page) = compute_list_options::<UserModel>(Some(list_options))?;
/// ```
pub fn compute_list_options<MC: DMC>(
    list_options: Option<ListOptions>,
) -> Result<(ListOptions, u64)> {
    // Step 1: Get list options or use defaults
    let mut list_options = list_options.unwrap_or_default();

    // Step 2: Set and validate limit
    let limit = list_options
        .limit
        .unwrap_or(LIST_LIMIT_DEFAULT)
        .min(LIST_LIMIT_MAX);
    list_options.limit = Some(limit);

    // Step 3: Calculate current page based on offset and limit
    let offset = list_options.offset.unwrap_or(0).max(0);
    let limit = list_options.limit.unwrap_or(LIST_LIMIT_DEFAULT) as f64;
    let page = if offset == 0 { 1 } else { ((offset as f64) / limit).ceil() as u64 + 1 };

    Ok((list_options, page))
}

/// Updates multiple records in the database based on a list of IDs
///
/// # Arguments
/// * `db` - The database connection manager
/// * `ids` - Vector of record IDs to update
/// * `input` - The data to update the records with
///
/// # Returns
/// * `Result<()>` - Success if all records were updated, Error if any record was not found
///
/// # Example
/// ```rust
/// let ids = vec![user_id1, user_id2];
/// let input = UpdateUserInput { status: "active".to_string() };
/// update_many::<UserModel, _>(db, ids, input).await?;
/// ```
pub async fn update_many<MC, I>(db: &ModelManager, ids: Vec<Uuid>, input: I) -> Result<()>
where
    MC: DMC,
    I: HasSeaFields,
{
    // Step 1: Extract non-null fields and prepare for update
    let fields = input.not_none_sea_fields();
    let fields = fields.for_sea_update();

    // Step 2: Build UPDATE query for multiple records
    let mut query = Query::update();
    query
        .table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(MC::ID).is_in(ids.clone()));

    // Step 3: Execute query and check if any records were updated
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_with(&sql, values);
    let result = db.dbx().execute(sqlx_query).await?;

    if result == 0 {
        Err(Error::EntityNotFound { entity: MC::TABLE, id: 0 })
    } else {
        Ok(())
    }
}

/// Checks if any record exists in the database matching the given filter
///
/// # Arguments
/// * `db` - The database connection manager
/// * `filter` - Optional filter conditions to check for existence
///
/// # Returns
/// * `Result<bool>` - True if any matching record exists, false otherwise
///
/// # Example
/// ```rust
/// let filter = UserFilter { email: Some("user@example.com".to_string()) };
/// let exists = exists::<UserModel, _>(db, Some(filter)).await?;
/// ```
pub async fn exists<MC, F>(db: &ModelManager, filter: Option<F>) -> Result<bool>
where
    MC: DMC,
    F: Into<FilterGroups>,
{
    // Step 1: Build simple SELECT query with limit 1
    let mut query = Query::select();
    query.from(MC::table_ref()).expr(Expr::val(1)).limit(1);

    // Step 2: Apply filter conditions if provided
    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond);
    }

    // Step 3: Execute query and check if any record exists
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result: Option<i32> = sqlx::query_scalar_with(&sql, values)
        .fetch_optional(db.dbx().db())
        .await?;

    Ok(result.is_some())
}

/// Retrieves multiple records by their IDs
///
/// # Arguments
/// * `db` - The database connection manager
/// * `ids` - Vector of record IDs to retrieve
///
/// # Returns
/// * `Result<Vec<O>>` - Vector of found records
///
/// # Example
/// ```rust
/// let ids = vec![user_id1, user_id2];
/// let users = find_by_ids::<UserModel, User>(db, ids).await?;
/// ```
pub async fn find_by_ids<MC, O>(db: &ModelManager, ids: Vec<Uuid>) -> Result<Vec<O>>
where
    MC: DMC,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Step 1: Early return if no IDs provided
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    // Step 2: Build SELECT query for multiple IDs
    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(O::sea_column_refs())
        .and_where(Expr::col(MC::ID).is_in(ids));

    // Step 3: Execute query and get results
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);
    let entities = db.dbx().fetch_all(sqlx_query).await?;

    Ok(entities)
}

/// Updates records matching the given filter conditions
///
/// # Arguments
/// * `db` - The database connection manager
/// * `filter` - Filter conditions to match records for update
/// * `input` - The data to update the matching records with
///
/// # Returns
/// * `Result<u64>` - Number of records updated
///
/// # Example
/// ```rust
/// let filter = UserFilter { status: Some("inactive".to_string()) };
/// let input = UpdateUserInput { status: "active".to_string() };
/// let updated_count = update_by_filter::<UserModel, _, _>(db, filter, input).await?;
/// ```
pub async fn update_by_filter<MC, I, F>(db: &ModelManager, filter: F, input: I) -> Result<u64>
where
    MC: DMC,
    I: HasSeaFields,
    F: Into<FilterGroups>,
{
    // Step 1: Extract non-null fields and prepare for update
    let fields = input.not_none_sea_fields();
    let fields = fields.for_sea_update();

    // Step 2: Build UPDATE query with filter conditions
    let mut query = Query::update();
    query.table(MC::table_ref()).values(fields);

    // Step 3: Apply filter conditions
    let filters: FilterGroups = filter.into();
    let cond: Condition = filters.try_into()?;
    query.cond_where(cond);

    // Step 4: Execute query and return number of updated records
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_with(&sql, values);
    let result = db.dbx().execute(sqlx_query).await?;

    Ok(result)
}

/// Trait for converting between Rust enums and PostgreSQL enum types
pub trait EnumConverter {
    /// Convert a Rust enum value to a PostgreSQL enum string
    fn to_pg_enum(&self) -> String;

    /// Convert a PostgreSQL enum string to a Rust enum value
    fn from_pg_enum(value: &str) -> Self;
}

/// Enhanced builder for handling PostgreSQL enum types
pub struct PostgresEnumQueryBuilder {
    /// Cache for compiled regex patterns
    pattern_cache: HashMap<String, Regex>,
    /// Cache for enum type mappings
    enum_type_cache: HashMap<String, String>,
}

impl PostgresEnumQueryBuilder {
    /// Creates a new instance of PostgresEnumQueryBuilder
    pub fn new() -> Self {
        Self {
            pattern_cache: HashMap::new(),
            enum_type_cache: HashMap::new(),
        }
    }

    /// Builds a SQL query with proper enum type casting and optimized performance
    /// 
    /// # Arguments
    /// * `query` - The insert statement to process
    /// * `enum_columns` - List of column names that are enum types
    /// 
    /// # Returns
    /// * `Result<(String, SqlxValues), Error>` - The processed SQL query and values, or an error
    pub fn build_sqlx_with_enum_cast(
        &mut self,
        query: &sea_query::InsertStatement,
        enum_columns: &[&str],
    ) -> Result<(String, SqlxValues)> {
        // Validate input
        if enum_columns.is_empty() {
            return Err(Error::InvalidEnumValue { value: "No enum columns provided".to_string() });
        }

        // Build base query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        
        // Extract column information efficiently
        let (column_names, returning_columns) = self.extract_columns(&sql)?;
        
        // Pre-compile regex patterns for better performance
        let patterns = self.compile_patterns(&column_names, &returning_columns);
        
        // Process values with optimized enum handling
        let (custom_values, param_index) = self.process_values(&values, &column_names, enum_columns)?;
        
        // Apply type casting with caching
        let final_sql = self.apply_type_casting(
            &sql,
            &patterns,
            &column_names,
            &returning_columns,
            enum_columns,
            param_index,
        )?;

        // Debug logging in debug mode
        #[cfg(debug_assertions)]
        {
            println!("Final SQL: {}", final_sql);
            println!("Values: {:?}", custom_values);
        }

        Ok((final_sql, SqlxValues(sea_query::Values(custom_values))))
    }

    /// Compiles regex patterns for efficient matching
    fn compile_patterns(
        &mut self,
        column_names: &[&str],
        returning_columns: &[&str],
    ) -> HashMap<String, Regex> {
        let mut patterns = HashMap::new();
        
        // Compile patterns for column names
        for column in column_names {
            let pattern = format!("\"{}\"", column);
            if !self.pattern_cache.contains_key(&pattern) {
                let regex = Regex::new(&pattern).expect("Invalid regex pattern");
                self.pattern_cache.insert(pattern.clone(), regex);
            }
            if let Some(regex) = self.pattern_cache.get(&pattern) {
                patterns.insert(pattern, regex.clone());
            }
        }
        
        // Compile patterns for returning columns
        for column in returning_columns {
            let pattern = format!("\"{}\"", column);
            if !self.pattern_cache.contains_key(&pattern) {
                let regex = Regex::new(&pattern).expect("Invalid regex pattern");
                self.pattern_cache.insert(pattern.clone(), regex);
            }
            if let Some(regex) = self.pattern_cache.get(&pattern) {
                patterns.insert(pattern, regex.clone());
            }
        }
        
        patterns
    }

    /// Processes values with optimized enum handling
    fn process_values(
        &self,
        values: &SqlxValues,
        column_names: &[&str],
        enum_columns: &[&str],
    ) -> Result<(Vec<Value>, i32)> {
        let mut custom_values = Vec::new();
        let mut param_index = 1;

        for (i, value) in values.0.iter().enumerate() {
            match value {
                Value::String(Some(s)) => {
                    if let Some((_, enum_value)) = s.split_once("::") {
                        // Handle explicit enum casting
                        custom_values.push(Value::String(Some(Box::new(enum_value.to_string()))));
                    } else if let Some(column_name) = column_names.get(i) {
                        // Handle implicit enum casting
                        if enum_columns.contains(column_name) {
                            custom_values.push(value.clone());
                        } else {
                            custom_values.push(value.clone());
                        }
                    } else {
                        custom_values.push(value.clone());
                    }
                }
                _ => custom_values.push(value.clone()),
            }
            param_index += 1;
        }

        Ok((custom_values, param_index))
    }

    /// Applies type casting with caching
    fn apply_type_casting(
        &mut self,
        sql: &str,
        patterns: &HashMap<String, Regex>,
        column_names: &[&str],
        returning_columns: &[&str],
        enum_columns: &[&str],
        param_index: i32,
    ) -> Result<String> {
        let mut final_sql = sql.to_string();
        
        // Apply type casting for enum columns in VALUES clause
        for (i, column) in column_names.iter().enumerate() {
            if enum_columns.contains(column) {
                let enum_type = self.get_enum_type(column)?;
                let param_num = i + 1;
                let pattern = format!("${}", param_num);
                let replacement = format!("${}::{}", param_num, enum_type);
                final_sql = final_sql.replace(&pattern, &replacement);
            }
        }
        
        // Apply type casting for enum columns in RETURNING clause
        for column in returning_columns {
            if enum_columns.contains(column) {
                let enum_type = self.get_enum_type(column)?;
                let pattern = format!("\"{}\"", column);
                let replacement = format!("\"{}\"::{}", column, enum_type);
                final_sql = final_sql.replace(&pattern, &replacement);
            }
        }
        
        Ok(final_sql)
    }

    /// Gets enum type with caching
    fn get_enum_type(&mut self, column: &str) -> Result<String> {
        if let Some(cached_type) = self.enum_type_cache.get(column) {
            return Ok(cached_type.clone());
        }
        
        // Use the column name directly as the enum type name
        let enum_type = column.to_lowercase();
        self.enum_type_cache.insert(column.to_string(), enum_type.clone());
        
        Ok(enum_type)
    }

    /// Extract column names and returning columns from SQL query
    fn extract_columns<'a>(&self, sql: &'a str) -> Result<(Vec<&'a str>, Vec<&'a str>)> {
        let column_names = sql
            .split("(\"")
            .nth(1)
            .and_then(|s| s.split("\")").next())
            .map(|s| s.split("\", \"").collect())
            .ok_or_else(|| Error::InvalidEnumValue { value: "Invalid SQL format".to_string() })?;

        let returning_columns = sql
            .split("RETURNING \"")
            .nth(1)
            .and_then(|s| s.split("\"").next())
            .map(|s| s.split("\", \"").collect())
            .ok_or_else(|| Error::InvalidEnumValue { value: "Invalid SQL format".to_string() })?;

        Ok((column_names, returning_columns))
    }
}

/// Creates a record with proper enum handling
pub async fn create_with_enum_cast<MC, I, O>(db: &ModelManager, input: I) -> Result<O>
where
    MC: DMC,
    I: HasSeaFields,
    O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
    // Step 1: Extract non-null fields and prepare for insertion
    let fields = input.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    // Step 2: Build and validate the INSERT query
    let mut query = Query::insert();
    query
        .into_table(MC::table_ref())
        .columns(columns)
        .values(sea_values)?;

    // Step 3: Add RETURNING clause with proper column selection
    let o_fields = O::sea_column_refs();
    query.returning(Query::returning().columns(o_fields));

    // Step 4: Build SQL with enum casting
    let mut builder = PostgresEnumQueryBuilder::new();
    let (sql, values) = builder.build_sqlx_with_enum_cast(&query, MC::ENUM_COLUMNS)?;

    // Step 5: Log the generated SQL for debugging (only in debug mode)
    #[cfg(debug_assertions)]
    {
        println!("Generated SQL with casts: {}", sql);
        println!("Values: {:?}", values);
    }

    // Step 6: Execute query with proper error handling
    let sqlx_query = sqlx::query_as_with::<_, O, _>(&sql, values);

    match db.dbx().fetch_one(sqlx_query).await {
        Ok(entity) => Ok(entity),
        Err(e) => match e {
            jd_storage::dbx::Error::Sqlx(sqlx_err) => {
                // Handle specific database errors
                if let Some(db_err) = sqlx_err.as_database_error() {
                    match db_err.code().as_deref() {
                        Some("23505") => Err(Error::UniqueViolation {
                            table: db_err.table().unwrap_or("unknown").to_string(),
                            constraint: db_err.constraint().unwrap_or("unknown").to_string(),
                        }),
                        Some("22P02") => {
                            Err(Error::InvalidEnumValue { value: db_err.message().to_string() })
                        }
                        Some("42703") => {
                            Err(Error::ColumnNotFound { column: db_err.message().to_string() })
                        }
                        _ => Err(Error::Sqlx(sqlx_err)),
                    }
                } else {
                    Err(Error::Sqlx(sqlx_err))
                }
            }
            _ => Err(Error::Dbx(e)),
        },
    }
}
