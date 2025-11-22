mod core;
mod queue;
mod router;
mod tasks;
mod types;
mod utils;
mod worker;
use std::{
    env,
    sync::{Arc, Mutex},
};

use axum::Router;
use tokio::sync::Semaphore;
use urlencoding::encode;

use crate::{
    core::database::database_core,
    router::export_router::ExportRouter,
    types::{AppState, ExportJob, Message},
    worker::{cleaning_file_worker::cleaning_file, worker::Worker},
};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Build DB URL
    let db_password = encode("w2%$b6TQFcS5#JpDL4G");
    let db_username = env::var("DATABASE_USERNAME").unwrap();
    let db_name = env::var("DATABASE_NAME").unwrap();
    let db_host = env::var("DATABASE_HOST").unwrap();
    let db_port = env::var("DATABASE_PORT").unwrap();

    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_username, db_password, db_host, db_port, db_name
    );

    println!("URL DEBUG = {:?}", database_url);

    // init database
    let database = database_core(&database_url).await.unwrap();

    // channel untuk worker
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(100);

    // state hanya butuh tx + db
    let state = Arc::new(AppState {
        db: database.clone(),
        tx: tx.clone(),
    });

    let worker_service = Worker::new(state.clone());
    let semaphore = Arc::new(Semaphore::new(1));
    // Worker thread
    tokio::spawn(async move {
        let mut jobs: Vec<ExportJob> = Vec::new();

        while let Some(msg) = rx.recv().await {
            match msg {
                Message::Add(job) => {
                    jobs.push(job.clone());

                    let mut worker_clone = worker_service.clone();
                    let job_clone = job.clone();
                    let tx_clone = tx.clone();
                    let semaphore_clone = semaphore.clone();
                    //worker init terpisah
                    tokio::spawn(async move {
                        let _permit = semaphore_clone.acquire().await;
                        let total_data = worker_clone.fetch_total_data_candidate(&job);
                        let _ = tx_clone
                            .send(Message::InitChunk {
                                id: job_clone.id.clone().unwrap(),
                                total_chunk: total_data.await.unwrap_or(0) / 2000,
                            })
                            .await;
                        worker_clone.run_worker_init_data().await;
                        worker_clone.init_worker(&job_clone).await;

                        // Setelah selesai kirim update ke main loop
                        let _ = tx_clone
                            .send(Message::UpdateSuccess {
                                id: job_clone.id.clone().unwrap(),
                            })
                            .await;
                    });
                }

                /// init chunk
                Message::InitChunk { id, total_chunk } => {
                    if let Some(j) = jobs.iter_mut().find(|j| j.id.as_ref() == Some(&id)) {
                        j.total_chunk = Some(total_chunk);
                    }
                }

                // update chunk
                Message::UpdateChunk { id, chunk } => {
                    if let Some(j) = jobs.iter_mut().find(|j| j.id.as_ref() == Some(&id)) {
                        j.current_chunk = Some(chunk);
                    }
                }

                // update success
                Message::UpdateSuccess { id } => {
                    if let Some(j) = jobs.iter_mut().find(|j| j.id.as_ref() == Some(&id)) {
                        j.status = types::ProcessStatus::SUCCESS;
                    }

                    cleaning_file(&id, "output.xlsx").expect("Erro cleaning file csv");
                }

                // get list reply
                Message::List(reply) => {
                    let _ = reply.send(jobs.clone());
                }
            }
        }
    });

    // Routes
    let export_router = ExportRouter::new(state.clone());
    let app = Router::new().merge(export_router.build_router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5559").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
