use sea_query::{Query, PostgresQueryBuilder, SqlxValues};

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