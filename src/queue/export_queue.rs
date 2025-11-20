use std::{
    fmt::Error,
    sync::{
        Arc,
        mpsc::{self, channel},
    },
    thread,
    time::SystemTime,
};

use anyhow::Result;
use serde::Serialize;
use uuid::Uuid;

use crate::types::AppState;

// use crate::worker::fetch::FetchWorker;

pub struct ExportQueue {
    state: Arc<AppState>,
}

impl ExportQueue {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state: state }
    }

    pub fn queue(&self) {}
}
