use std::{collections::HashMap, fs::File, io::BufReader};

use axum_prometheus::PrometheusMetricLayerBuilder;
use openmensa_parser_darmstadt_server::{AppState, feed};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    canteens: HashMap<String, Vec<String>>,
    deploy_url: Option<String>,
    bind: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let config_path = match std::env::args().nth(1) {
        Some(v) => v,
        None => "config.json".into(),
    };
    let file = File::open(config_path)
        .expect("failed to open config file, please set the path accordingly");
    let reader = BufReader::new(file);
    let config: Config =
        serde_json::from_reader(reader).expect("failed to deserialize: invalid config file");

    let mut registered_canteens = HashMap::new();
    for (id, identifiers) in config.canteens {
        for identifier in identifiers {
            registered_canteens.insert(identifier, id.clone());
        }
    }

    let prometheus_prefix =
        std::env::var("PROMETHEUS_PREFIX").unwrap_or(std::env!("CARGO_PKG_NAME").into());
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix(prometheus_prefix)
        .enable_response_body_size(true)
        .with_default_metrics()
        .build_pair();

    let app = axum::Router::new()
        .nest("/feed", feed::router())
        .route(
            "/metrics",
            axum::routing::get(|| async move { metric_handle.render() }),
        )
        .with_state(AppState {
            deploy_url: config.deploy_url,
            registered_canteens,
        })
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(prometheus_layer);

    let bind = config.bind.unwrap_or("0.0.0.0:3000".into());
    tracing::info!("binding to {}", bind);
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
