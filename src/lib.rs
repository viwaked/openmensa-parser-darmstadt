use std::collections::HashMap;

pub mod feed;
pub mod graphql;
pub mod openmensa;
pub mod parser;

#[derive(Clone)]
pub struct AppState {
    pub deploy_url: Option<String>,
    pub registered_canteens: HashMap<String, String>, // identifier:canteenId
}
