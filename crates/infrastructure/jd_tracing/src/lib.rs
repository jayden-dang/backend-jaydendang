use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// TODO: telemetry -> Tracing microservices
pub fn tracing_init() -> Result<()> {
    color_eyre::install()?;

    // Tạo log filter từ biến môi trường hoặc fallback về INFO
    let env_filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(ErrorLayer::default())
        .with(
            fmt::layer()
                .without_time() // Only Dev
                .pretty() // hoặc dùng `.json()` nếu muốn output JSON
                .with_target(true) // in tên target/module
                .with_thread_names(true), // in tên thread
        )
        .init();
    Ok(())
}
