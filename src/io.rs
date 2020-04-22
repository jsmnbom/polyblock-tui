use std::sync::Arc;
use tokio::sync::Mutex;

use crate::App;

#[derive(Debug)]
pub enum IoEvent {}

#[derive(Clone)]
pub struct Io<'a> {
    app: &'a Arc<Mutex<App>>,
}

impl<'a> Io<'a> {
    pub fn new(app: &'a Arc<Mutex<App>>) -> Self {
        Io { app }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        match io_event {}
    }
}
