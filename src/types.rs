use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, prelude::FromRow};
use tokio::sync::{mpsc::Sender, oneshot};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ExportRequest {
    pub vacancy_id: Option<i64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub email: String,
}
#[derive(Debug, Serialize, Clone, Deserialize)]
pub enum ProcessStatus {
    Success,
    Failed,
    Process,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJob {
    pub id: Option<String>,
    pub status: ProcessStatus,
    pub total_chunk: Option<i32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub vacancy_id: Option<i64>,
    pub final_output: Option<String>,
    pub client_email: String,
    pub current_chunk: Option<i32>,
    pub is_candidate_pool: bool,
    pub employer_id: i64,
    pub total: i32,
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
    InitChunk { id: String, total_chunk: i32, total: i32 },
    List(oneshot::Sender<Vec<super::ExportJob>>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailTemplate {
    pub logo_url: String,
    pub subject: String,
    pub body: String,
    pub donwload_url: Option<String>,
    pub client_email: String,
}

//  return {

//       "status": "ok",
//       "message": "Application is healthy",
//       "timestamp": now.isoformat(),
//       "date": now.strftime("%Y-%m-%d %H:%M:%S")

//     }

#[derive(Serialize, Deserialize, Debug)]
pub struct Healthz {
    pub status: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub date: String,
}

impl Default for Healthz {
    fn default() -> Self {
        Self {
            status: "Ok".to_string(),
            message: "Application is healthy".to_string(),
            timestamp: Utc::now(),
            date: Utc::now().format("%Y-%m-%d").to_string(),
        }
    }
}
