use std::sync::Arc;

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

        let query =
            String::from("SELECT COUNT(DISTINCT vacancy_can_id) from _temp_taekwang_export");
        let query_sql = format!("{} where {}", query, filters.join(" AND "));

        let count = sqlx::query_scalar(&query_sql)
            .fetch_one(&self.state.db.clone())
            .await?;
        Ok(count)
    }

    pub async fn fetch_candidate_data(
        &self,
        cursor_id: Option<u64>,
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
}
