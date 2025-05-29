use crate::{
    application::use_cases::sui_use_cases::SuiUseCases,
    domain::sui_repository_trait::SuiRepository,
    models::{CoinBalance, DynamicFieldPage, ObjectInfo, requests::*},
    Result,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress, TransactionDigest},
    event::{Event, EventFilter, EventPage},
};

pub struct SuiHandler<R: SuiRepository> {
    use_cases: SuiUseCases<R>,
}

impl<R: SuiRepository> SuiHandler<R> {
    pub fn new(use_cases: SuiUseCases<R>) -> Self {
        Self { use_cases }
    }

    // Read operations
    pub async fn get_object(&self, Json(req): Json<GetObjectRequest>) -> Result<Json<ObjectInfo>> {
        let object = self.use_cases.get_object(req.object_id).await?;
        Ok(Json(object))
    }

    pub async fn get_coin_balance(&self, Json(req): Json<GetCoinBalanceRequest>) -> Result<Json<CoinBalance>> {
        let balance = self.use_cases.get_coin_balance(req.address, &req.coin_type).await?;
        Ok(Json(balance))
    }

    pub async fn get_dynamic_fields(&self, Json(req): Json<GetDynamicFieldsRequest>) -> Result<Json<DynamicFieldPage>> {
        let fields = self.use_cases.get_dynamic_fields(req.parent_object_id, req.cursor, req.limit).await?;
        Ok(Json(fields))
    }

    // Event operations
    pub async fn get_events(&self, Json(req): Json<GetEventsRequest>) -> Result<Json<EventPage>> {
        let events = self.use_cases.get_events(req.filter, req.cursor, req.limit, req.descending_order).await?;
        Ok(Json(events))
    }

    pub async fn get_events_by_transaction(&self, Json(req): Json<GetEventsByTransactionRequest>) -> Result<Json<Vec<Event>>> {
        let events = self.use_cases.get_events_by_transaction(req.digest).await?;
        Ok(Json(events))
    }

    pub async fn get_events_by_module(&self, Json(req): Json<GetEventsByModuleRequest>) -> Result<Json<EventPage>> {
        let events = self.use_cases.get_events_by_module(req.package, req.module, req.cursor, req.limit).await?;
        Ok(Json(events))
    }
}