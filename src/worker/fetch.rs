use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::Error;

use crate::types::{AppState, ExportJob, RowData};
// use xlsxwriter::worksheet::filter;

#[derive(Clone)]
pub struct FetchWorker {
    state: Arc<AppState>,
}

impl FetchWorker {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn fetch_total_candidate(&self, candidate_option: &ExportJob) -> Result<i32, Error> {
        let mut filters: Vec<String> = Vec::new();

        filters.push(format!("employer_id = {} ", &candidate_option.employer_id));

        if let Some(vacancy_id) = &candidate_option.vacancy_id {
            filters.push(format!("vacancy_id = {}", vacancy_id));
        }

        let query = String::from("SELECT COUNT(DISTINCT vacancy_can_id) from _temp_taekwang_export");
        let query_sql = format!("{} where {}", query, filters.join(" AND "));

        let count = sqlx::query_scalar(&query_sql).fetch_one(&self.state.db.clone()).await?;
        Ok(count)
    }

    pub async fn fetch_candidate_pool(
        &self,
        cursor: &Option<u64>,
        export: &ExportJob,
    ) -> Result<Option<Vec<RowData>>, sqlx::Error> {
        let mut filters = Vec::<String>::new();

        filters.push(String::from(format!("tte.employer_id = {}", export.employer_id)));

        if let (Some(start_date), Some(end_date)) = (export.start_date, export.end_date) {
            // let format_start_date: DateTime<Utc> = start_date.into();
            // let format_end_date: DateTime<Utc> = end_date.into();
            let string_query = format!(
                "tte.applied_date between {} and {}",
                start_date.format("%Y-%m-%d").to_string(),
                end_date.format("%Y-%m-%d").to_string()
            );
            filters.push(string_query);
        }

        if let Some(last_id) = cursor {
            filters.push(format!("tte.vacancy_can_id > {}", last_id));
        }

        if let Some(vacancy_id) = export.vacancy_id {
            filters.push(format!("tte.vacancy_id = {}", vacancy_id));
        }

        let query_select_max_join = "JOIN(SELECT candidate_id, max(id) as max_id from _temp_taekwang_export GROUP BY candidate_id)latest on latest.max_id = tte.id and latest.candidate_id = tte.candidate_id";

        let mut where_builder = String::new();

        if filters.len() > 0 {
            where_builder = format!("WHERE {}", filters.join(" AND "));
        }

        let query_sql = format!(
            "
             select
            distinct tte.candidate_id as candidate_id,
            tte.candidate as full_name,
            tte.email as email,
            tte.phone as phone,
            tte.id_number as id_number,
            tte.whatsapp as wa_number,
            tte.license_info as license_info,
            tte.expected_salary as expected_salary,
            tte.latest_vacancy as latest_vacancy,
            tte.education_history as education_history,
            tte.work_experience as work_experience,
            tte.language_skills as language_skills,
            tte.certifications as certifications,
            tte.id as id
            from _temp_taekwang_export tte
            {}
            {}
            order by tte.id asc
            LIMIT 2000
            ",
            query_select_max_join, where_builder
        );

        let records = sqlx::query_as::<_, RowData>(&query_sql)
            .fetch_all(&self.state.db)
            .await?;
        Ok(Some(records))
    }
    pub async fn candidate_management(
        &self,
        cursor_id: &Option<u64>,
        candidate_option: &ExportJob,
    ) -> Result<Option<Vec<RowData>>, Error> {
        let mut filters: Vec<String> = Vec::new();

        filters.push(format!("employer_id = {} ", candidate_option.employer_id));

        if let Some(vacancy_id) = &candidate_option.vacancy_id {
            filters.push(format!(" vacancy_id = {}", vacancy_id));
        }

        if let Some(id) = cursor_id {
            filters.push(format!(" tte.id > {}", id));
        }

        let query = format!(
            "
            select
            distinct tt.vacancy_can_id,
            tt.candidate_id as candidate_id,
            tt.candidate as full_name,
            tt.email as email,
            tt.phone as phone,
            tt.id_number as id_number,
            tt.whatsapp as wa_number,
            tt.license_info as license_info,
            tt.expected_salary as expected_salary,
            tt.latest_vacancy as latest_vacancy,
            tt.education_history as education_history,
            tt.work_experience as work_experience,
            tt.vacancy as vacancy,
            tt.applied_date as applied_date,
            tt.language_skills as language_skills,
            tt.certifications as certifications,
            tt.id as id
            from _temp_taekwang_export tt
            JOIN (
                SELECT MIN(id) AS min_id
                FROM _temp_taekwang_export tte
                where {}
                GROUP BY vacancy_can_id
                ) sub ON sub.min_id = tt.id
            order by tt.id asc
            LIMIT {}
        ",
            filters.join(" AND "),
            2000
        );

        let records: Vec<RowData> = sqlx::query_as::<_, RowData>(query.as_str())
            .fetch_all(&self.state.db)
            .await?;

        Ok(Some(records))
    }

    pub async fn run_candidate_fetch(
        &self,
        last_vacancy_id: &Option<u64>,
        export: &ExportJob,
    ) -> Result<Option<Vec<RowData>>, sqlx::Error> {
        let mut records: Option<Vec<RowData>> = None;

        if export.is_candidate_pool {
            records = self.fetch_candidate_pool(last_vacancy_id, export).await?;
        } else {
            records = self.candidate_management(last_vacancy_id, export).await?;
        }

        Ok(records)
    }
}
