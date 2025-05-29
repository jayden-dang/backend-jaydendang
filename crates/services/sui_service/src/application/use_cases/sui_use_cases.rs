use crate::domain::sui_repository_trait::SuiRepository;

pub struct SuiUseCases<R: SuiRepository> {
  pub repository: R,
}

impl<R: SuiRepository> SuiUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  // Read operations
  // pub async fn get_object(&self, object_id: ObjectID) -> Result<ObjectInfo> {
  //     self.repository.get_object(object_id).await
  // }
}
