// -->>> Region:: START  --->>>  Public Modules
pub mod application;
pub mod infrastructure;
pub mod models;
// <<<-- Region:: END    <<<---  Public Modules

mod domain;
mod error;

use application::{handlers::sui_handler::SuiHandler, use_cases::sui_use_cases::SuiUseCases};
use error::Error;
use infrastructure::sui_repository_impl::SuiRepositoryImpl;
use jd_core::sui::sui_client::SuiClient;
type Result<T> = std::result::Result<T, Error>;

pub struct SuiService {
  handler: SuiHandler<SuiRepositoryImpl>,
}

impl SuiService {
  pub async fn new(client: SuiClient) -> Self {
    let repository = SuiRepositoryImpl::new(client);
    let use_cases = SuiUseCases::new(repository);
    let handler = SuiHandler::new(use_cases);

    Self { handler }
  }

  pub fn handler(&self) -> &SuiHandler<SuiRepositoryImpl> {
    &self.handler
  }
}
