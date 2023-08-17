use autometrics::autometrics;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use opentelemetry::metrics::MetricsError;
use opentelemetry::sdk::metrics::MeterProvider;
use opentelemetry::{runtime, Context};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use rand::{thread_rng, Rng};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

fn init_metrics() -> Result<MeterProvider, MetricsError> {
    let export_config = ExportConfig {
        endpoint: "http://ams.winter-sun-917.internal:4317".to_string(),
        ..ExportConfig::default()
    };
    let push_interval = Duration::from_secs(1);
    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .with_period(push_interval)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let meter_provider = init_metrics()?;
    let cx = Context::current();

    let app = Router::new()
        .route("/", get(index))
        .route("/slow", get(slow_function))
        .route("/error", get(error_function));

    let addr = "[::]:3000".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    println!("Waiting so that we could see metrics going down...");
    sleep(Duration::from_secs(10)).await;
    meter_provider.force_flush(&cx)?;

    Ok(())
}

// our main handler function that is fine
#[autometrics()]
pub async fn index() -> impl IntoResponse {
    return "Hello, World!";
}

// our slow function that is slow
#[autometrics()]
pub async fn slow_function() -> impl IntoResponse {
    sleep(Duration::from_millis(1000)).await;
    return "Hello, World again!";
}

// our error function that errors
#[autometrics()]
pub async fn error_function() -> Result<String, StatusCode> {
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
}

pub async fn sleep_random_duration() {
    let sleep_duration = Duration::from_millis(thread_rng().gen_range(0..300));
    sleep(sleep_duration).await;
}
