use color_eyre::eyre::Result;
use jd_utils::time;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
  EnvFilter,
  fmt::{self, format::FmtSpan, time::SystemTime},
  layer::{Layer, SubscriberExt},
  util::SubscriberInitExt,
};
/// Environment types for different deployment stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
  Development,
  Staging,
  Production,
  Testing,
}

impl Environment {
  /// Get environment from env var or default to Development
  pub fn from_env() -> Self {
    match env::var("ENVIRONMENT")
      .or_else(|_| env::var("ENV"))
      .unwrap_or_else(|_| "development".to_string())
      .to_lowercase()
      .as_str()
    {
      "production" | "prod" => Self::Production,
      "staging" | "stage" => Self::Staging,
      "testing" | "test" => Self::Testing,
      _ => Self::Development,
    }
  }

  /// Check if running in production
  pub fn is_production(&self) -> bool {
    matches!(self, Self::Production)
  }

  /// Check if debug logging should be enabled
  pub fn enable_debug(&self) -> bool {
    !matches!(self, Self::Production)
  }
}

/// Tracing configuration for different environments
#[derive(Debug, Clone)]
pub struct TracingConfig {
  pub environment: Environment,
  pub default_level: Level,
  pub enable_colors: bool,
  pub enable_timestamps: bool,
  pub use_json_format: bool,
  pub enable_thread_names: bool,
  pub enable_span_events: bool,
  pub custom_filter: Option<String>,
}

impl TracingConfig {
  /// Create config based on environment
  pub fn from_environment(env: Environment) -> Self {
    match env {
      Environment::Production => Self::production(),
      Environment::Staging => Self::staging(),
      Environment::Testing => Self::testing(),
      Environment::Development => Self::development(),
    }
  }

  /// Production configuration - minimal, structured logging
  pub fn production() -> Self {
    Self {
      environment: Environment::Production,
      default_level: Level::INFO,
      enable_colors: false,
      enable_timestamps: true,
      use_json_format: true,
      enable_thread_names: false,
      enable_span_events: false,
      custom_filter: None,
    }
  }

  /// Staging configuration - more detailed for debugging
  pub fn staging() -> Self {
    Self {
      environment: Environment::Staging,
      default_level: Level::DEBUG,
      enable_colors: false,
      enable_timestamps: true,
      use_json_format: true,
      enable_thread_names: true,
      enable_span_events: true,
      custom_filter: None,
    }
  }

  /// Development configuration - human-friendly, colorful
  pub fn development() -> Self {
    Self {
      environment: Environment::Development,
      default_level: Level::TRACE,
      enable_colors: true,
      enable_timestamps: false,
      use_json_format: false,
      enable_thread_names: false,
      enable_span_events: false,
      custom_filter: None,
    }
  }

  /// Testing configuration - minimal output
  pub fn testing() -> Self {
    Self {
      environment: Environment::Testing,
      default_level: Level::WARN,
      enable_colors: false,
      enable_timestamps: false,
      use_json_format: false,
      enable_thread_names: false,
      enable_span_events: false,
      custom_filter: Some("warn".to_string()),
    }
  }

  /// Create environment filter based on config
  pub fn create_env_filter(&self) -> Result<EnvFilter> {
    let base_filter = if let Some(custom) = &self.custom_filter {
      custom.clone()
    } else {
      match self.environment {
        Environment::Production => {
          // Only essential logs in production
          format!(
            "{},sqlx=warn,hyper=warn,tokio=warn,h2=warn,tower=warn,reqwest=warn,rustls=warn,jsonrpsee=warn",
            self.default_level.as_str().to_lowercase()
          )
        }
        Environment::Staging => {
          // More detailed for staging but filter noise
          format!(
            "{},sqlx=info,hyper=info,h2=warn,rustls=warn,jsonrpsee=info",
            self.default_level.as_str().to_lowercase()
          )
        }
        Environment::Testing => {
          // Minimal for tests
          "warn,jd_=error".to_string()
        }
        Environment::Development => {
          // Application logs at debug/trace, but external deps at info/warn to reduce noise
          "debug,jd_=trace,api_gateway=trace,user_service=trace,sui_service=trace,\
             hyper=warn,tokio=warn,tokio::runtime::worker=off,h2=warn,tower=warn,reqwest=info,\
             rustls=warn,jsonrpsee=info,jsonrpsee_http_client=warn,fastcrypto=warn,\
             auth_service::infrastructure::signature_verifier=debug,\
             api_gateway::middleware::mw_request_context=warn,\
             api_gateway::middleware::mw_res_map=info,\
             api_gateway::log=info"
            .to_string()
        }
      }
    };

    Ok(
      EnvFilter::builder()
        .with_default_directive(self.default_level.into())
        .parse_lossy(&base_filter),
    )
  }
}

/// Initialize tracing with improved configuration
pub fn tracing_init() -> Result<()> {
  tracing_init_with_config(TracingConfig::from_environment(Environment::from_env()))
}

/// Initialize tracing with custom configuration
pub fn tracing_init_with_config(config: TracingConfig) -> Result<()> {
  color_eyre::install()?;

  let env_filter = config.create_env_filter()?;

  let fmt_layer = fmt::layer()
    .with_target(false)
    .with_thread_names(false)
    .with_span_events(if config.enable_span_events { FmtSpan::CLOSE } else { FmtSpan::NONE })
    .with_timer(SystemTime)
    .with_file(false)
    .with_line_number(false);

  let fmt_layer: Box<dyn Layer<_> + Send + Sync> = if config.use_json_format {
    Box::new(fmt_layer.json())
  } else {
    let mut layer = fmt_layer.pretty();
    if !config.enable_colors {
      layer = layer.with_ansi(false);
    }
    Box::new(layer)
  };

  tracing_subscriber::registry()
    .with(env_filter)
    .with(ErrorLayer::default())
    .with(fmt_layer)
    .init();

  // Log initialization info with our custom time format
  tracing::info!(
      environment = ?config.environment,
      level = ?config.default_level,
      json_format = config.use_json_format,
      timestamp = time::format_time(time::now_utc()),
      "Tracing initialized"
  );

  Ok(())
}

/// Initialize tracing specifically for tests
pub fn tracing_init_test() -> Result<()> {
  tracing_init_with_config(TracingConfig::testing())
}
