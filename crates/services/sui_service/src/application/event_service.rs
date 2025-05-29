use crate::error::Error;
use crate::Result;
use sui_sdk::types::base_types::{ObjectID, TransactionDigest};

pub struct EventService {
  client: SuiClient,
}

impl EventService {
  pub fn new(client: SuiClient) -> Self {
    Self { client }
  }

  pub async fn get_events(
    &self,
    filter: EventFilter,
    cursor: Option<TransactionDigest>,
    limit: Option<usize>,
    descending_order: bool,
  ) -> Result<EventPage> {
    self
      .client
      .client
      .event_api()
      .get_events(filter, cursor, limit, descending_order)
      .await
      .map_err(|e| Error::SuiClientError(e.to_string()))
  }

  pub async fn get_events_by_transaction(
    &self,
    digest: TransactionDigest,
  ) -> Result<Vec<sui_sdk::types::event::Event>> {
    self
      .client
      .client
      .event_api()
      .get_events_by_transaction(digest)
      .await
      .map_err(|e| Error::SuiClientError(e.to_string()))
  }

  pub async fn get_events_by_module(
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
