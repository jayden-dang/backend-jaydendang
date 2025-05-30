use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;

use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiObjectResponseQuery};
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::{
  base_types::{ObjectID, SuiAddress},
  gas_coin::GasCoin,
};

use crate::models::GasPoolStatus;

#[derive(Debug)]
pub struct GasObject {
  pub object_id: ObjectID,
  pub version: u64,
  pub balance: u64,
  pub in_use: bool,
}

pub struct GasStation {
  pub sui_client: SuiClient,
  pub sponsor_address: SuiAddress,
  pub gas_pool: RwLock<HashMap<ObjectID, GasObject>>,
  pub max_gas_budget: u64,
}

impl GasStation {
  pub async fn new(
    sui_rpc_url: &str,
    sponsor_address: SuiAddress,
    max_gas_budget: u64,
  ) -> Result<Self> {
    // Initialize SUI client
    let sui_client = SuiClientBuilder::default().build(sui_rpc_url).await?;

    // Initialize gas pool
    let gas_station =
      Self { sui_client, sponsor_address, gas_pool: RwLock::new(HashMap::new()), max_gas_budget };

    // Load gas objects
    gas_station.refresh_gas_pool().await?;

    Ok(gas_station)
  }

  pub async fn refresh_gas_pool(&self) -> Result<()> {
    let gas_objects = self
      .sui_client
      .read_api()
      .get_owned_objects(
        self.sponsor_address,
        Some(SuiObjectResponseQuery::new_with_options(
          SuiObjectDataOptions::new().with_type().with_content(),
        )),
        None,
        None,
      )
      .await?;

    let mut pool = self.gas_pool.write().await;
    pool.clear();

    for obj_response in gas_objects.data {
      if let Some(obj) = obj_response.data {
        if let Ok(gas_coin) = GasCoin::try_from(&obj) {
          let gas_object = GasObject {
            object_id: obj.object_id,
            version: obj.version.value(),
            balance: gas_coin.value(),
            in_use: false,
          };
          pool.insert(obj.object_id, gas_object);
        }
      }
    }

    info!("Refreshed gas pool with {} objects", pool.len());
    Ok(())
  }

  pub async fn get_available_gas(&self, required_budget: u64) -> Result<ObjectID> {
    let mut pool = self.gas_pool.write().await;

    for (object_id, gas_obj) in pool.iter_mut() {
      if !gas_obj.in_use && gas_obj.balance >= required_budget {
        gas_obj.in_use = true;
        return Ok(*object_id);
      }
    }

    anyhow::bail!("No available gas object with sufficient balance");
  }

  pub async fn release_gas(&self, object_id: ObjectID) {
    let mut pool = self.gas_pool.write().await;
    if let Some(gas_obj) = pool.get_mut(&object_id) {
      gas_obj.in_use = false;
    }
  }

  pub async fn get_pool_stats(&self) -> GasPoolStatus {
    let pool = self.gas_pool.read().await;
    let total_objects = pool.len();
    let total_balance: u64 = pool.values().map(|obj| obj.balance).sum();
    let available_objects = pool.values().filter(|obj| !obj.in_use).count();

    GasPoolStatus {
      total_objects,
      total_balance,
      available_objects,
      utilization_rate: if total_objects > 0 {
        (total_objects - available_objects) as f64 / total_objects as f64 * 100.0
      } else {
        0.0
      },
    }
  }
}
