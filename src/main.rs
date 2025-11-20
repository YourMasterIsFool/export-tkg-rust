mod core;
mod queue;
mod router;
mod tasks;
mod types;
mod utils;
mod worker;
use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
};

use axum::Router;
use urlencoding::encode;

use crate::{
    core::database::database_core,
    router::export_router::ExportRouter,
    types::{AppState, ExportJob},
    worker::{
        excel_worker::{self, excel_worker_fn},
        fetch::{FetchCandidate, FetchWorker},
        worker::Worker,
    },
};

#[tokio::main]
async fn main() {
    // excel_worker_fn("csv/fc9f8127-82b4-4a69-8f44-e5f15f6f8b42");
    let password = encode("w2%$b6TQFcS5#JpDL4G");
    let username = "karirpad_v5";
    let database_url = format!(
        "mysql://{}:{}@dbconn.jobseeker.software:13306/karirpad_dev",
        username, password
    );
    println!("URL DEBUG = {:?}", database_url);

    // init database
    let database = database_core(&database_url).await.unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ExportJob>(100);

    let jobs = Arc::new(Mutex::new(Vec::<ExportJob>::new()));
    let state = Arc::new(AppState {
        db: database,
        tx: tx.clone(),
        jobs: jobs.clone(),
    });

    let fetch_worker = FetchWorker::new(state.clone());
    // start worker

    let worker = Worker::new(state.clone());

    tokio::spawn(async move {
        while let Some(job) = rx.recv().await {
            worker.init_worker(&job).await.unwrap();
        }
    });

    let export_router = ExportRouter::new(state.clone());

    let app = Router::new().merge(export_router.build_router());

    // axum server

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5559").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
