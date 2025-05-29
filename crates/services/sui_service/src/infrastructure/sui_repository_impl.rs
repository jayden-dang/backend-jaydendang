use crate::{
    domain::sui_repository_trait::SuiRepository,
    error::Error,
    models::{CoinBalance, DynamicFieldPage, ObjectInfo},
    Result,
};
use async_trait::async_trait;
use jd_core::sui::SuiClient;
use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress, TransactionDigest},
    dynamic_field::DynamicFieldInfo,
    event::{Event, EventFilter, EventPage},
    object::ObjectRead,
};

pub struct SuiRepositoryImpl {
    client: SuiClient,
}

impl SuiRepositoryImpl {
    pub fn new(client: SuiClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SuiRepository for SuiRepositoryImpl {
    async fn get_object(&self, object_id: ObjectID) -> Result<ObjectInfo> {
        let object_read = self
            .client
            .client
            .read_api()
            .get_object_with_options(object_id, None)
            .await
            .map_err(|e| Error::SuiClientError(e.to_string()))?;

        match object_read {
            ObjectRead::Exists(object) => Ok(ObjectInfo {
                object_id: object.object_id,
                version: object.version,
                digest: object.digest.to_string(),
                type_: object.type_.to_string(),
                owner: object.owner.to_string(),
                previous_transaction: object.previous_transaction.to_string(),
                storage_rebate: object.storage_rebate,
                content: object.data,
            }),
            _ => Err(Error::InvalidRequest("Object not found".to_string())),
        }
    }

    async fn get_coin_balance(&self, address: SuiAddress, coin_type: &str) -> Result<CoinBalance> {
        let balance = self
            .client
            .client
            .coin_read_api()
            .get_balance(address, Some(coin_type.to_string()))
            .await
            .map_err(|e| Error::SuiClientError(e.to_string()))?;

        Ok(CoinBalance {
            coin_type: balance.coin_type.to_string(),
            coin_object_count: balance.coin_object_count,
            total_balance: balance.total_balance,
            locked_balance: balance.locked_balance,
        })
    }

    async fn get_dynamic_fields(
        &self,
        parent_object_id: ObjectID,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
    ) -> Result<DynamicFieldPage> {
        let page = self
            .client
            .client
            .read_api()
            .get_dynamic_fields(parent_object_id, cursor, limit)
            .await
            .map_err(|e| Error::SuiClientError(e.to_string()))?;

        Ok(DynamicFieldPage {
            data: page.data,
            next_cursor: page.next_cursor,
            has_next_page: page.has_next_page,
        })
    }

    async fn get_events(
        &self,
        filter: EventFilter,
        cursor: Option<TransactionDigest>,
        limit: Option<usize>,
        descending_order: bool,
    ) -> Result<EventPage> {
        self.client
            .client
            .event_api()
            .get_events(filter, cursor, limit, descending_order)
            .await
            .map_err(|e| Error::SuiClientError(e.to_string()))
    }

    async fn get_events_by_transaction(&self, digest: TransactionDigest) -> Result<Vec<Event>> {
        self.client
            .client
            .event_api()
            .get_events_by_transaction(digest)
            .await
            .map_err(|e| Error::SuiClientError(e.to_string()))
    }

    async fn get_events_by_module(
        &self,
        package: ObjectID,
        module: String,
        cursor: Option<TransactionDigest>,
        limit: Option<usize>,
    ) -> Result<EventPage> {
        let filter = EventFilter::Module { package, module };
        self.get_events(filter, cursor, limit, true).await
    }
} 