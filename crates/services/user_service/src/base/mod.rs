use modql::SIden;
use sea_query::{Iden, SeaRc, TableRef};
use serde::Serialize;

#[derive(Iden)]
pub enum ComminId {
    UserId,
    OwnerId,
}

#[derive(Serialize)]
pub struct PaginationMetadata {}

#[derive(Iden)]
pub enum TimestampIden {
    Cid,
    Ctime,
    Mid,
    Mtime,
}

pub trait DBbmc {
    const SCHEMA: &'static str;
    const TABLE: &'static str;

    fn table_ref() -> TableRef {
        TableRef::SchemaTable(SeaRc::new(SIden(Self::SCHEMA)), SeaRc::new(SIden(Self::TABLE)))
    }

    fn has_timestamp() -> bool {
        true
    }

    fn has_owner_id() -> bool {
        false
    }
}
