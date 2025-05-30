use modql::SIden;
use sea_query::{Iden, SeaRc, TableRef};
use serde::Serialize;

pub mod bmc_macros;
pub mod error;
pub mod handlers;
pub mod rest;
pub mod rpc;

// -->>> Region:: START  --->>>  Constants
const LIST_LIMIT_DEFAULT: i64 = 20;
const LIST_LIMIT_MAX: i64 = 50;
// <<<-- Region:: END    <<<---  Constants

#[derive(Iden)]
pub enum CommonId {
  Id,
  OwnerId,
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
  total_items: u64,
  total_pages: u64,
}

pub trait DMC {
  const SCHEMA: &'static str;
  const TABLE: &'static str;
  const ID: &'static str;
  const ENUM_COLUMNS: &'static [&'static str];

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
