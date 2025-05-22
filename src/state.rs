use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub name: String,
    pub id: u64,
}

pub struct AppState {
    pub config: Arc<Config>,

    pub places: DashMap<String, Place>,
    pub active_place: TokioMutex<Option<String>>,

    pub only_log_failures: bool,
    pub debug: bool,
}
