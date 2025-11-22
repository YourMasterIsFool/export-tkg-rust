use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, mpsc},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, prelude::FromRow};
use tokio::sync::{mpsc::Sender, oneshot};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ExportRequest {
    pub vacancy_id: Option<i64>,
    pub start_date: Option<SystemTime>,
    pub end_date: Option<SystemTime>,
    pub is_candidate_management: bool,
    pub email: String,
}
#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum ProcessStatus {
    SUCCESS,
    FAILED,
    PROCESS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJob {
    pub id: Option<String>,
    pub status: ProcessStatus,
    pub total_chunk: Option<i32>,
    pub start_date: Option<SystemTime>,
    pub end_date: Option<SystemTime>,
    pub vacancy_id: Option<i64>,
    pub final_output: Option<String>,
    pub client_email: String,
    pub current_chunk: Option<i32>,
    pub is_candidate_pool: bool,
    pub employer_id: i64,
}

#[derive(Debug)]
pub struct JobQueue {
    pub id: String,
    pub status: ProcessStatus,
    pub total_chunk: Option<i32>,
    pub final_output: Option<String>,
    pub client_email: String,
}
#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub tx: Sender<Message>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct RowData {
    pub id: u64,
    pub candidate_id: i64,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub id_number: Option<String>,
    pub wa_number: Option<String>,
    pub license_info: Option<String>,
    pub expected_salary: Option<String>,
    // pub latest_vacancy: DateTi,
    pub education_history: Option<String>,
    pub work_experience: Option<String>,
    pub vacancy: Option<String>,
    pub applied_date: DateTime<Utc>,
    pub language_skills: Option<String>,
    pub certifications: Option<String>,
}

impl RowData {
    pub fn to_hashmap(&self) -> Result<HashMap<String, String>, serde_json::Error> {
        let json = serde_json::to_value(self)?;

        let mut map = HashMap::new();
        if let serde_json::Value::Object(obj) = json {
            for (key, value) in obj {
                let string_value = match value {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Null => "".to_string(),
                    _ => value.to_string(),
                };
                map.insert(key, string_value);
            }
        }

        Ok(map)
    }
}

#[derive(Debug)]
pub enum Message {
    Add(super::ExportJob),
    UpdateSuccess { id: String },
    UpdateChunk { id: String, chunk: i32 },
    InitChunk { id: String, total_chunk: i32 },
    List(oneshot::Sender<Vec<super::ExportJob>>),
}
