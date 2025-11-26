use std::sync::Arc;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get, routing::post};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::types::{AppState, ExportJob, ExportRequest, Message, ProcessStatus};

pub struct ExportRouter {
    state: Arc<AppState>,
}

impl Default for ExportJob {
    fn default() -> Self {
        Self {
            client_email: String::from("tkg.dev@yopmail.com"),
            final_output: None,
            id: None,
            status: ProcessStatus::Process,
            total_chunk: Some(0),
            current_chunk: Some(0),
            is_candidate_pool: false,
            employer_id: 63402,
            end_date: None,
            vacancy_id: Some(333438),
            start_date: None,
            total: 0,
            expired_at: None,
        }
    }
}
impl ExportRouter {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub fn build_router(&self) -> Router {
        Router::new()
            .nest(
                "/export",
                Router::new()
                    .route("/candidate-management", post(Self::export_candidate_management))
                    .route("/candidate-pool", post(Self::export_candidate_pool))
                    .route("/status", get(Self::list_status)),
            )
            .with_state(self.state.clone())
    }

    pub async fn export_candidate_management(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<ExportRequest>,
    ) -> impl IntoResponse {
        let id = Uuid::new_v4();
        let job = ExportJob {
            id: Some(String::from(id)),
            client_email: payload.email,
            is_candidate_pool: false,
            vacancy_id: payload.vacancy_id,
            expired_at: Some(Utc::now() + Duration::days(2)),

            ..Default::default()
        };
        let message = Message::Add(job.clone());
        let tx = state.tx.clone();

        tx.send(message)
            .await
            .map_err(|err| println!("failed send job: {}", err))
            .unwrap();

        Json(job).into_response()
    }

    pub async fn export_candidate_pool(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<ExportRequest>,
    ) -> impl IntoResponse {
        let id = Uuid::new_v4();
        let job = ExportJob {
            id: Some(String::from(id)),
            client_email: payload.email,
            is_candidate_pool: true,
            start_date: payload.start_date,
            end_date: payload.end_date,
            expired_at: Some(Utc::now() + Duration::days(2)),
            ..Default::default()
        };
        let message = Message::Add(job.clone());
        let tx = state.tx.clone();

        tx.send(message).await.expect("cannot add job");

        Json(job).into_response()
    }

    pub async fn list_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        let (tx_reply, rx_reply) = tokio::sync::oneshot::channel();

        state.tx.send(Message::List(tx_reply)).await.unwrap();
        let jobs = rx_reply.await.unwrap();

        Json(jobs)
    }
}
