use sea_query::{Query, PostgresQueryBuilder, SqlxValues};
use std::collections::HashMap;
use regex::Regex;

/// Information about enum columns for automatic casting
#[derive(Debug, Clone)]
pub struct EnumColumnInfo {
    pub column_name: String,
    pub postgres_type: String,
}

/// Trait for models that have enum columns
pub trait HasEnumColumns {
    fn has_enum_columns() -> bool { 
        false 
    }
    
    fn enum_column_info() -> Vec<EnumColumnInfo> { 
        vec![] 
    }
}

/// Smart query builder that handles enum casting automatically
pub struct EnumAwareQueryBuilder;

impl EnumAwareQueryBuilder {
    pub fn build_sqlx(
        query: &sea_query::InsertStatement,
        enum_columns: Vec<EnumColumnInfo>,
    ) -> (String, SqlxValues) {
        let (mut sql, values) = query.build_sqlx(PostgresQueryBuilder);
        
        // Simple approach: replace parameters with enum casts
        // This is a basic implementation - you can make it more sophisticated
        for (i, enum_info) in enum_columns.iter().enumerate() {
            let param_num = i + 2; // Start from $2 (assuming $1 is user_id)
            let old_param = format!("${}", param_num);
            let new_param = format!("${}::{}", param_num, enum_info.postgres_type);
            
            // Only replace if this parameter exists and corresponds to an enum column
            if sql.contains(&old_param) {
                sql = sql.replace(&old_param, &new_param);
            }
        }
        
        (sql.to_string(), values)
    }
    
    /// More sophisticated version that analyzes column positions
    pub fn build_sqlx_with_column_analysis(
        query: &sea_query::InsertStatement,
        enum_columns: Vec<EnumColumnInfo>,
    ) -> (String, SqlxValues) {
        let (mut sql, values) = query.build_sqlx(PostgresQueryBuilder);
        
        // Extract column order from INSERT statement
        if let Some(columns_part) = Self::extract_insert_columns(&sql) {
            let column_names: Vec<&str> = columns_part
                .split(", ")
                .map(|s| s.trim_matches('"'))
                .collect();
            
            // Map enum columns to their parameter positions
            for (pos, column_name) in column_names.iter().enumerate() {
                if let Some(enum_info) = enum_columns.iter()
                    .find(|info| info.column_name == *column_name) {
                    
                    let param_num = pos + 1;
                    let old_param = format!("${}", param_num);
                    let new_param = format!("${}::{}", param_num, enum_info.postgres_type);
                    sql = sql.replace(&old_param, &new_param);
                }
            }
        }
        
        (sql.to_string(), values)
    }
    
    /// Extract column names from INSERT statement
    fn extract_insert_columns(sql: &str) -> Option<&str> {
        sql.split(" (")
            .nth(1)?
            .split(") VALUES")
            .next()
    }
}

/// Error handling utilities
#[derive(Debug)]
pub enum EnumError {
    InvalidValue(String),
    TypeMismatch(String),
    Unknown(String),
}

impl std::fmt::Display for EnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnumError::InvalidValue(msg) => write!(f, "Invalid enum value: {}", msg),
            EnumError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            EnumError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for EnumError {}

/// A builder for handling PostgreSQL enum types with optimized performance and error handling
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
    /// * `Result<(String, SqlxValues), EnumError>` - The processed SQL query and values, or an error
    /// 
    /// # Example
    /// ```rust
    /// let builder = PostgresEnumQueryBuilder::new();
    /// let (sql, values) = builder.build_sqlx_with_enum_cast(&query, &["status", "type"])?;
    /// ```
    pub fn build_sqlx_with_enum_cast(
        &mut self,
        query: &sea_query::InsertStatement,
        enum_columns: &[&str],
    ) -> Result<(String, SqlxValues), EnumError> {
        // Validate input
        if enum_columns.is_empty() {
            return Err(EnumError::InvalidValue("No enum columns provided".to_string()));
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
                self.pattern_cache.insert(
                    pattern.clone(),
                    Regex::new(&pattern).expect("Invalid regex pattern"),
                );
            }
            patterns.insert(pattern, self.pattern_cache.get(&pattern).unwrap().clone());
        }
        
        // Compile patterns for returning columns
        for column in returning_columns {
            let pattern = format!("\"{}\"", column);
            if !self.pattern_cache.contains_key(&pattern) {
                self.pattern_cache.insert(
                    pattern.clone(),
                    Regex::new(&pattern).expect("Invalid regex pattern"),
                );
            }
            patterns.insert(pattern, self.pattern_cache.get(&pattern).unwrap().clone());
        }
        
        patterns
    }

    /// Processes values with optimized enum handling
    fn process_values(
        &self,
        values: &[Value],
        column_names: &[&str],
        enum_columns: &[&str],
    ) -> Result<(Vec<Value>, i32), EnumError> {
        let mut custom_values = Vec::with_capacity(values.len());
        let mut param_index = 1;

        for (i, value) in values.iter().enumerate() {
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
    ) -> Result<String, EnumError> {
        let mut final_sql = sql.to_string();
        
        // Apply type casting for enum columns in VALUES clause
        for column in column_names {
            if enum_columns.contains(column) {
                let enum_type = self.get_enum_type(column)?;
                let pattern = format!("${}", param_index);
                let replacement = format!("(${}::TEXT)::{}", param_index, enum_type);
                final_sql = final_sql.replace(&pattern, &replacement);
            }
        }
        
        // Apply type casting for enum columns in RETURNING clause
        for column in returning_columns {
            if enum_columns.contains(column) {
                let enum_type = self.get_enum_type(column)?;
                let pattern = format!("\"{}\"", column);
                let replacement = format!("(\"{}\"::TEXT)::{}", column, enum_type);
                final_sql = final_sql.replace(&pattern, &replacement);
            }
        }
        
        Ok(final_sql)
    }

    /// Gets enum type with caching
    fn get_enum_type(&mut self, column: &str) -> Result<String, EnumError> {
        if let Some(cached_type) = self.enum_type_cache.get(column) {
            return Ok(cached_type.clone());
        }
        
        let enum_type = format!("{}_enum", column.to_lowercase());
        self.enum_type_cache.insert(column.to_string(), enum_type.clone());
        
        Ok(enum_type)
    }

    /// Extract column names and returning columns from SQL query
    fn extract_columns(&self, sql: &str) -> Result<(Vec<&str>, Vec<&str>), EnumError> {
        let column_names = sql
            .split("(\"")
            .nth(1)
            .and_then(|s| s.split("\")").next())
            .map(|s| s.split("\", \"").collect())
            .ok_or_else(|| EnumError::InvalidValue("Invalid SQL format".to_string()))?;

        let returning_columns = sql
            .split("RETURNING \"")
            .nth(1)
            .and_then(|s| s.split("\"").next())
            .map(|s| s.split("\", \"").collect())
            .ok_or_else(|| EnumError::InvalidValue("Invalid SQL format".to_string()))?;

        Ok((column_names, returning_columns))
    }
}