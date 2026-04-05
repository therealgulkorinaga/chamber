use chambers_runtime::Runtime;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type AppState = Arc<Mutex<Runtime>>;
