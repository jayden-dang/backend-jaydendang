use crate::domain::sui_repository_trait::SuiRepository;
use crate::{Result, error::Error};
use sui_sdk::rpc_types::{SuiObjectResponse, SuiObjectDataOptions, Page};
use sui_sdk::rpc_types::DynamicFieldInfo;
use sui_sdk::types::base_types::SuiAddress;
use sui_types::base_types::ObjectID;
use sui_types::object::Owner;
use std::str::FromStr;

/// Use cases for object operations on Sui blockchain
#[derive(Clone)]
pub struct ObjectUseCases<R: SuiRepository> {
  repository: R,
}

impl<R: SuiRepository> ObjectUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  /// Get detailed object information
  pub async fn get_object_details(&self, object_id: &str) -> Result<SuiObjectResponse> {
    let obj_id = ObjectID::from_str(object_id)
      .map_err(|_| Error::InvalidRequest("Invalid object ID format".to_string()))?;

    let options = Some(SuiObjectDataOptions {
      show_type: true,
      show_owner: true,
      show_previous_transaction: true,
      show_display: true,
      show_content: true,
      show_bcs: false,
      show_storage_rebate: true,
    });

    self.repository.get_object(obj_id, options).await
  }

  /// Get basic object information (minimal data)
  pub async fn get_object_basic(&self, object_id: &str) -> Result<SuiObjectResponse> {
    let obj_id = ObjectID::from_str(object_id)
      .map_err(|_| Error::InvalidRequest("Invalid object ID format".to_string()))?;

    let options = Some(SuiObjectDataOptions {
      show_type: true,
      show_owner: true,
      show_previous_transaction: false,
      show_display: false,
      show_content: false,
      show_bcs: false,
      show_storage_rebate: false,
    });

    self.repository.get_object(obj_id, options).await
  }

  /// Get multiple objects at once
  pub async fn get_objects_batch(&self, object_ids: Vec<String>) -> Result<Vec<SuiObjectResponse>> {
    let obj_ids: Result<Vec<ObjectID>> = object_ids
      .into_iter()
      .map(|id| {
        ObjectID::from_str(&id)
          .map_err(|_| Error::InvalidRequest(format!("Invalid object ID: {}", id)))
      })
      .collect();

    let obj_ids = obj_ids?;

    let options = Some(SuiObjectDataOptions {
      show_type: true,
      show_owner: true,
      show_previous_transaction: true,
      show_display: true,
      show_content: true,
      show_bcs: false,
      show_storage_rebate: true,
    });

    self.repository.get_objects(obj_ids, options).await
  }

  /// Get all objects owned by an address
  pub async fn get_owned_objects(
    &self,
    address: &str,
    object_type: Option<String>,
    limit: Option<usize>,
  ) -> Result<Page<SuiObjectResponse, ObjectID>> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;

    self.repository
      .get_owned_objects(sui_address, object_type, None, limit)
      .await
  }

  /// Get objects owned by address with specific type filter
  pub async fn get_owned_objects_by_type(
    &self,
    address: &str,
    object_type: &str,
    limit: Option<usize>,
  ) -> Result<Page<SuiObjectResponse, ObjectID>> {
    self.get_owned_objects(address, Some(object_type.to_string()), limit).await
  }

  /// Get dynamic fields of an object
  pub async fn get_dynamic_fields(
    &self,
    parent_object_id: &str,
    limit: Option<usize>,
  ) -> Result<Page<DynamicFieldInfo, ObjectID>> {
    let obj_id = ObjectID::from_str(parent_object_id)
      .map_err(|_| Error::InvalidRequest("Invalid parent object ID format".to_string()))?;

    self.repository.get_dynamic_fields(obj_id, None, limit).await
  }

  /// Check if an object exists and is accessible
  pub async fn object_exists(&self, object_id: &str) -> Result<bool> {
    match self.get_object_basic(object_id).await {
      Ok(response) => Ok(response.data.is_some()),
      Err(Error::InvalidRequest(_)) => Ok(false),
      Err(e) => Err(e),
    }
  }

  /// Get object owner information
  pub async fn get_object_owner(&self, object_id: &str) -> Result<Option<ObjectOwner>> {
    let response = self.get_object_basic(object_id).await?;
    
    if let Some(data) = response.data {
      if let Some(owner) = data.owner {
        return Ok(Some(ObjectOwner::from_sui_owner(owner)));
      }
    }
    
    Ok(None)
  }

}

/// Object owner information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ObjectOwner {
  AddressOwner(String),
  ObjectOwner(String),
  Shared,
  Immutable,
}

impl ObjectOwner {
  fn from_sui_owner(owner: Owner) -> Self {
    match owner {
      Owner::AddressOwner(addr) => ObjectOwner::AddressOwner(addr.to_string()),
      Owner::ObjectOwner(obj_id) => ObjectOwner::ObjectOwner(obj_id.to_string()),
      Owner::Shared { .. } => ObjectOwner::Shared,
      Owner::Immutable => ObjectOwner::Immutable,
      Owner::ConsensusAddressOwner { .. } => ObjectOwner::AddressOwner("consensus".to_string()),
    }
  }
}

