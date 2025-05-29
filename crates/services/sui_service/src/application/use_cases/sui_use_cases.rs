use crate::{
    domain::sui_repository_trait::SuiRepository,
    models::{CoinBalance, DynamicFieldPage, ObjectInfo},
    Result,
};
use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress, TransactionDigest},
    event::{Event, EventFilter, EventPage},
};

pub struct SuiUseCases<R: SuiRepository> {
    repository: R,
}

impl<R: SuiRepository> SuiUseCases<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    // Read operations
    pub async fn get_object(&self, object_id: ObjectID) -> Result<ObjectInfo> {
        self.repository.get_object(object_id).await
    }

    pub async fn get_coin_balance(&self, address: SuiAddress, coin_type: &str) -> Result<CoinBalance> {
        self.repository.get_coin_balance(address, coin_type).await
    }

    pub async fn get_dynamic_fields(
        &self,
        parent_object_id: ObjectID,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
    ) -> Result<DynamicFieldPage> {
        self.repository.get_dynamic_fields(parent_object_id, cursor, limit).await
    }

    // Event operations
    pub async fn get_events(
        &self,
        filter: EventFilter,
        cursor: Option<TransactionDigest>,
        limit: Option<usize>,
        descending_order: bool,
    ) -> Result<EventPage> {
        self.repository.get_events(filter, cursor, limit, descending_order).await
    }

    pub async fn get_events_by_transaction(&self, digest: TransactionDigest) -> Result<Vec<Event>> {
        self.repository.get_events_by_transaction(digest).await
    }

    pub async fn get_events_by_module(
        &self,
        package: ObjectID,
        module: String,
        cursor: Option<TransactionDigest>,
        limit: Option<usize>,
    ) -> Result<EventPage> {
        self.repository.get_events_by_module(package, module, cursor, limit).await
    }
} 