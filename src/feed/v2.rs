use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing,
};

use crate::{
    AppState,
    openmensa::{self, OpenMensa},
    parser::fetch_openmensa_for_range,
};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/{identifier}/full.xml", routing::get(get_full))
        .route("/{identifier}/today.xml", routing::get(get_today))
}

fn openmensa_to_response(mensa_data: &OpenMensa) -> Response {
    match mensa_data.serialize_to_string() {
        Ok(body) => {
            let mut response = Response::new(body.into());
            response
                .headers_mut()
                .insert("Content-Type", HeaderValue::from_static("application/xml"));

            response
        }
        Err(e) => {
            tracing::error!("failed to serialize openmensa data: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_full(State(state): State<AppState>, Path(identifier): Path<String>) -> Response {
    let canteen_id = match state.registered_canteens.get(&identifier) {
        Some(id) => id,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let today = chrono::Local::now().date_naive();
    match fetch_openmensa_for_range(canteen_id.clone(), Some(today), None).await {
        Ok(mut data) => {
            if let Some(deploy_url) = state.deploy_url {
                data.canteen.feeds.push(openmensa::Feed {
                    name: "full".into(),
                    priority: Some(1),
                    url: format!("{}/feed/v2/{}/full.xml", deploy_url, identifier),
                    source: None,
                    schedule: Some(openmensa::Schedule {
                        day_of_month: Some("*".into()),
                        day_of_week: Some("*".into()),
                        hour: "4".into(),
                        retry: Some("60 5 1440".into()),
                        minute: None,
                        month: None,
                    }),
                });
                data.canteen.feeds.push(openmensa::Feed {
                    name: "today".into(),
                    priority: Some(0),
                    url: format!("{}/feed/v2/{}/today.xml", deploy_url, identifier),
                    source: None,
                    schedule: Some(openmensa::Schedule {
                        day_of_month: Some("*".into()),
                        day_of_week: Some("*".into()),
                        hour: "6-16".into(),
                        retry: Some("30 1".into()),
                        minute: None,
                        month: None,
                    }),
                });
            }
            openmensa_to_response(&data)
        }
        Err(e) => {
            tracing::error!("failed to fetch openmensa data: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_today(State(state): State<AppState>, Path(identifier): Path<String>) -> Response {
    let canteen_id = match state.registered_canteens.get(&identifier) {
        Some(id) => id,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let today = chrono::Local::now().date_naive();
    match fetch_openmensa_for_range(canteen_id.clone(), Some(today), Some(today)).await {
        Ok(mut data) => {
            if let Some(deploy_url) = state.deploy_url {
                data.canteen.feeds.push(openmensa::Feed {
                    name: "full".into(),
                    priority: Some(1),
                    url: format!("{}/feed/v2/{}/full.xml", deploy_url, identifier),
                    source: None,
                    schedule: Some(openmensa::Schedule {
                        day_of_month: Some("*".into()),
                        day_of_week: Some("*".into()),
                        hour: "8".into(),
                        retry: Some("60 5 1440".into()),
                        minute: None,
                        month: None,
                    }),
                });
                data.canteen.feeds.push(openmensa::Feed {
                    name: "today".into(),
                    priority: Some(0),
                    url: format!("{}/feed/v2/{}/today.xml", deploy_url, identifier),
                    source: None,
                    schedule: Some(openmensa::Schedule {
                        day_of_month: Some("*".into()),
                        day_of_week: Some("*".into()),
                        hour: "6-16".into(),
                        retry: Some("30 1".into()),
                        minute: None,
                        month: None,
                    }),
                });
            }
            openmensa_to_response(&data)
        }
        Err(e) => {
            tracing::error!("failed to fetch openmensa data: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
