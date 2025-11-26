mod core;
mod router;
mod tasks;
mod types;
mod utils;
mod worker;
use std::{env, sync::Arc};

use axum::routing::get;
use axum::{Json, Router, response::IntoResponse};
use tokio::sync::Semaphore;
use urlencoding::encode;

use crate::types::EmailTemplate;
use crate::worker::email_worker::EmailWorker;
use crate::worker::upload_worker::{self, upload_worker};
use crate::{
    core::database::database_core,
    router::export_router::ExportRouter,
    types::{AppState, ExportJob, Healthz, Message},
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

    let email_worker = EmailWorker::new("src/email_templates/**/*");
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
                        let total_data = worker_clone.fetch_total_data_candidate(&job).await.unwrap_or(0);
                        let _ = tx_clone
                            .send(Message::InitChunk {
                                id: job_clone.id.clone().unwrap(),
                                total_chunk: &total_data / 2000,
                                total: total_data,
                            })
                            .await;
                        match worker_clone.run_worker_init_data().await {
                            Ok(_value) => {
                                println!("worker data init successfully running")
                            }
                            Err(err) => println!("failed init worker data : {}", err),
                        }
                        match worker_clone.init_worker(&job_clone).await {
                            Ok(_value) => {
                                println!("running worker init successfully")
                            }
                            Err(err) => println!("failed init worker  : {}", err),
                        }

                        // Setelah selesai kirim update ke main loop
                        tx_clone
                            .send(Message::UpdateSuccess {
                                id: job_clone.id.clone().unwrap(),
                            })
                            .await
                            .expect("cannot update message to success");
                    });
                }

                // init chunk
                Message::InitChunk { id, total_chunk, total } => {
                    if let Some(j) = jobs.iter_mut().find(|j| j.id.as_ref() == Some(&id)) {
                        j.total_chunk = Some(total_chunk);
                        j.total = total
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
                        j.status = types::ProcessStatus::Success;
                        j.current_chunk = j.total_chunk;
                        let email_worker_clone = email_worker.clone();

                        let title = if j.is_candidate_pool {
                            "Export complete - candidate Pool"
                        } else {
                            "Export complete - candidate management"
                        }
                        .to_string();

                        let file_name_upload = if j.is_candidate_pool {
                            "candidate_pool"
                        } else {
                            "candidate_management"
                        };
                        match upload_worker(file_name_upload).await {
                            Ok(val) => {
                                println!("successflly upload s3");
                                let email_template = EmailTemplate {
                                    body: format!("export complete {} total data candidate : {}", title, j.total),
                                    subject: title,
                                    client_email: j.client_email.clone(),
                                    download_url: Some(val.clone()),
                                    logo_url: "".to_string(),
                                };

                                match email_worker_clone.send_success(&email_template) {
                                    Ok(()) => println!("Successfully uploaded email"),
                                    Err(err) => {
                                        println!("error upload email {}", err);
                                    }
                                }
                                j.final_output = Some(val.clone());
                            }
                            Err(err) => {
                                println!("error uploading : {}", err)
                            }
                        };
                        match cleaning_file(&id, "output.xlsx") {
                            Ok(()) => println!("successfully cleaning file"),
                            Err(err) => println!("failed cleaning file {}", err),
                        };
                    }
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
    let app = Router::new()
        .merge(export_router.build_router())
        .route("/healthz", get(router_healthz));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5559").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// healthz check for ci cd
async fn router_healthz() -> impl IntoResponse {
    let response = Healthz { ..Default::default() };

    Json(response).into_response()
}
