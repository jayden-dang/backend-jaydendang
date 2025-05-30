// -->>> Region:: START  --->>>  Public Modules
pub mod application;
pub mod infrastructure;
pub mod models;
// <<<-- Region:: END    <<<---  Public Modules

mod domain;
mod error;

use application::{handlers::sui_handler::SuiHandler, use_cases::sui_use_cases::SuiUseCases};
use error::Error;
use infrastructure::enhanced_sui_repository::EnhancedSuiRepository;
use jd_core::AppState;
type Result<T> = std::result::Result<T, Error>;

pub struct SuiService {
  handler: SuiHandler<EnhancedSuiRepository>,
}

impl SuiService {
  pub async fn new(state: AppState) -> Self {
    let repository = EnhancedSuiRepository::new(state);
    let use_cases = SuiUseCases::new(repository);
    let handler = SuiHandler::new(use_cases);

    Self { handler }
  }

  pub fn handler(&self) -> &SuiHandler<EnhancedSuiRepository> {
    &self.handler
  }
}
