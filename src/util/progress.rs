use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct ProgressState {
    msg: String,
    value: u64,
    length: u64,
}

#[derive(Clone)]
pub struct Progress {
    state: Arc<RwLock<ProgressState>>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ProgressState {
                msg: String::new(),
                value: 0,
                length: 0,
            })),
        }
    }

    pub async fn set_msg<S: Into<String>>(&self, msg: S) {
        self.state.write().await.msg = msg.into();
    }

    pub async fn set_length(&self, length: u64) {
        self.state.write().await.length = length;
    }

    pub async fn inc(&self, val: u64) {
        self.state.write().await.value += val;
    }

    pub async fn inc_with_msg<S: Into<String>>(&self, val: u64, msg: S) {
        let mut state = self.state.write().await;
        state.value += val;
        state.msg = msg.into();
    }

    pub async fn get(&self) -> f64 {
        let state = self.state.read().await;
        if state.length == 0 {
            return 0.0;
        }
        state.value as f64 / state.length as f64
    }

    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        state.length = 0;
        state.value = 0;
        state.msg = String::new();
    }

    pub async fn get_msg(&self) -> String {
        self.state.read().await.msg.clone()
    }
}
