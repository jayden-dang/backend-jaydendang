use crate::Result;
use async_trait::async_trait;
use jd_core::sui::SuiClient;
use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress, TransactionDigest},
    dynamic_field::DynamicFieldInfo,
    event::{Event, EventFilter, EventPage},
    object::ObjectRead,
};
use crate::models::{CoinBalance, DynamicFieldPage, ObjectInfo};

#[async_trait]
pub trait SuiRepository: Send + Sync {
    // Read operations
    async fn get_object(&self, object_id: ObjectID) -> Result<ObjectInfo>;
    async fn get_coin_balance(&self, address: SuiAddress, coin_type: &str) -> Result<CoinBalance>;
    async fn get_dynamic_fields(
        &self,
        parent_object_id: ObjectID,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
    ) -> Result<DynamicFieldPage>;

    // Event operations
    async fn get_events(
        &self,
        filter: EventFilter,
        cursor: Option<TransactionDigest>,
        limit: Option<usize>,
        descending_order: bool,
    ) -> Result<EventPage>;
    async fn get_events_by_transaction(&self, digest: TransactionDigest) -> Result<Vec<Event>>;
    async fn get_events_by_module(
        &self,
        package: ObjectID,
        module: String,
        cursor: Option<TransactionDigest>,
        limit: Option<usize>,
    ) -> Result<EventPage>;

    // API operations
    async fn get_api_version(&self) -> Result<String>;
    async fn get_events(&self, event_type: String) -> Result<Vec<String>>;
    async fn get_object(&self, object_id: String) -> Result<String>;
    async fn get_balance(&self, address: String) -> Result<String>;
} 