use std::{collections::HashMap, fs, sync::Arc};

use csv::{Error as CsvError, Writer};
use sqlx::Error;

use crate::{
    types::{AppState, ExportJob},
    utils::formatted_data::formatted_data,
    worker::{
        csv_worker::save_csv_worker,
        excel_worker::excel_worker_fn,
        fetch::{FetchCandidate, FetchWorker},
    },
};

use anyhow::{Ok as AyOke, Result as AnyResult};

pub struct Worker {
    state: Arc<AppState>,
}

impl Worker {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn init_worker(&self, job: &ExportJob) -> AnyResult<()> {
        self.fetch_worker(job).await.unwrap();
        excel_worker_fn(job.clone().id.unwrap().as_str());
        Ok(())
    }

    pub async fn fetch_worker(&self, job: &ExportJob) -> Result<(), Error> {
        let fetch = FetchWorker::new(self.state.clone());

        let mut chunk = 0;
        let mut last_id: Option<u64> = Some(0);

        println!("running worker fetch");
        loop {
            let fetch_data = fetch.fetch_candidate_data(last_id, &job).await?;
            println!("fetch data");
            println!("last_id: {:?}", last_id);
            if let Some(records) = fetch_data {
                if records.is_empty() {
                    break;
                }

                match records.last() {
                    Some(value) => {
                        last_id = Some(value.id);
                        chunk += 1;

                        let format_data = &records
                            .iter()
                            .map(|f| formatted_data(f.to_hashmap().unwrap())) // Flatten semua HashMap menjadi key-value pairs
                            .collect::<Vec<HashMap<String, String>>>();

                        println!("format data {:?}", format_data);

                        match save_csv_worker(
                            &job.id.clone().unwrap().as_str(),
                            &format!("{}", chunk),
                            format_data,
                        ) {
                            Ok(_) => {
                                println!("successfully created file csv");

                                self.update_chunk(job.id.clone(), chunk).await;
                            }
                            Err(err) => {
                                panic!("{}", err);
                            }
                        }
                    }
                    None => break,
                }

                // increment chunk
            } else {
                break;
            }
        }

        Ok(())
    }

    pub async fn update_chunk(&self, job_id: Option<String>, chunk: i32) {
        let mut jobs_lock = self.state.jobs.lock().unwrap();

        if let Some(job) = jobs_lock.iter_mut().find(|pre| pre.id == job_id) {
            job.total_chunk = Some(chunk);
            println!("update chunk");
        }
    }
}
