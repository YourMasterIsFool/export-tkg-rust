use std::{alloc::System, sync::Arc, time::SystemTime};

use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use uuid::Uuid;

use crate::types::{AppState, ExportJob, ExportRequest, JobQueue, ProcessStatus};

pub struct ExportRouter {
    state: Arc<AppState>,
}

impl Default for ExportJob {
    fn default() -> Self {
        Self {
            client_email: String::from("tkg.dev@yopmail.com"),
            final_output: None,
            id: None,
            status: ProcessStatus::PROCESS,
            total_chunk: Some(0),
            current_chunk: Some(0),
            is_candidate_pool: false,
            employer_id: 63402,
            end_date: None,
            vacancy_id: Some(333438),
            start_date: None,
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
                Router::new().route(
                    "/candidate_management",
                    post(Self::export_candidate_management),
                ),
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
            ..Default::default()
        };
        {
            let mut list = state.jobs.lock().unwrap();
            list.push(job.clone());
        }

        // kirim ke worker
        state.tx.send(job.clone()).await.unwrap();

        Json(job).into_response()
    }
}
