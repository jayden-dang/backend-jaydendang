use crate::{
  application::use_cases::sui_use_cases::SuiUseCases, domain::sui_repository_trait::SuiRepository,
};

pub struct SuiHandler<R: SuiRepository> {
  pub use_cases: SuiUseCases<R>,
}

impl<R: SuiRepository> SuiHandler<R> {
  pub fn new(use_cases: SuiUseCases<R>) -> Self {
    Self { use_cases }
  }

  // Read operations
  // pub async fn get_object(&self, Json(req): Json<GetObjectRequest>) -> Result<Json<ObjectInfo>> {
  //     let object = self.use_cases.get_object(req.object_id).await?;
  //     Ok(Json(object))
  // }
}
