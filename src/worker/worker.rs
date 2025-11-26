use std::{collections::HashMap, sync::Arc};

use rust_xlsxwriter::XlsxError;
use sqlx::Error;

use crate::{
    types::{AppState, ExportJob, Message},
    utils::formatted_data::formatted_data,
    worker::{
        csv_worker::save_csv_worker, excel_worker::excel_worker_fn, fetch::FetchWorker,
        init_data_worker::InitDataWorker,
    },
};

#[derive(Clone)]
pub struct Worker {
    state: Arc<AppState>,
    fetch_worker: FetchWorker,
    init_data_worker: InitDataWorker,
}

impl Worker {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state: state.clone(),
            fetch_worker: FetchWorker::new(state.clone()),
            init_data_worker: InitDataWorker::new(state.clone()),
        }
    }

    pub async fn run_worker_init_data(&mut self) -> Result<(), sqlx::Error> {
        match self.init_data_worker.run_worker().await {
            Ok(_) => {
                println!("susccess init data");
            }

            Err(err) => panic!("error init data {:?}", err),
        }

        // let last_vacancy_id = self.init_data_worker.get_last_id().await?;
        // println!(" last vacancy id: {}", last_vacancy_id.unwrap_or(0));
        Ok(())
    }
    pub async fn init_worker(&mut self, job: &ExportJob) -> Result<(), XlsxError> {
        match self.fetch_worker(job).await {
            Ok(_) => {
                println!("excel worker");
                excel_worker_fn(job.clone().id.unwrap().as_str()).await?;
            }
            Err(_) => {
                println!("err");
            }
        };

        Ok(())
    }
    pub async fn fetch_total_data_candidate(&self, job: &ExportJob) -> Result<i32, Error> {
        let total = &self.fetch_worker.fetch_total_candidate(job).await?;
        Ok(*total)
    }
    pub async fn fetch_worker(&self, job: &ExportJob) -> Result<(), Error> {
        let fetch = &self.fetch_worker;

        let mut chunk = 0;
        let mut last_id: Option<u64> = Some(0);

        println!("running worker fetch");
        loop {
            let fetch_data = fetch.run_candidate_fetch(&last_id, job).await?;
            println!("fetch data");
            println!("last_id: {:?}", last_id);
            if let Some(records) = fetch_data {
                if records.is_empty() {
                    match save_csv_worker(job.id.clone().unwrap().as_str(), &format!("{}", chunk), &None) {
                        Ok(_) => {
                            println!("successfully created file csv");

                            self.update_chunk(job.id.clone(), chunk).await;
                        }
                        Err(err) => {
                            panic!("{}", err);
                        }
                    }
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
                        match save_csv_worker(
                            job.id.clone().unwrap().as_str(),
                            &format!("{}", chunk),
                            &Some(format_data.to_vec()),
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
        let update = Message::UpdateChunk {
            id: job_id.unwrap_or("".to_string()),
            chunk,
        };

        let tx = &self.state.tx;

        tx.send(update).await.expect("cannot update chunks")
    }
}
