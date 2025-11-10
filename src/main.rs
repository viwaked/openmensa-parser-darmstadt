use std::{collections::HashMap, fs::File, io::BufReader};

use openmensa_darmstadt_parser::{AppState, feed};

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

    let app = axum::Router::new()
        .nest("/feed", feed::router())
        .with_state(AppState {
            deploy_url: config.deploy_url,
            registered_canteens,
        })
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let bind = config.bind.unwrap_or("0.0.0.0:3000".into());
    tracing::info!("binding to {}", bind);
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
