use std::sync::Arc;

use serde::Serialize;
use sqlx::{Row, prelude::FromRow};

use crate::types::AppState;

#[derive(Clone)]
pub struct InitDataWorker {
    candidate_ids: Vec<i32>,
    vacancies_id: Vec<i32>,

    state: Arc<AppState>,
}

#[derive(Serialize, FromRow, Debug)]
struct VacancyData {
    candidate_id: i32,
}
#[derive(Serialize, FromRow, Debug)]
struct VacanciesId {
    vacancy_can_id: i32,
}

const SEESION_EXPORT: &str = " SET SESSION group_concat_max_len = 1000000;";

impl InitDataWorker {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            candidate_ids: Vec::new(),
            vacancies_id: Vec::new(),
            state,
        }
    }

    pub fn chunk_list(&mut self) -> Vec<&[i32]> {
        let mut chunk_vacancies = Vec::new();

        // defautl offset
        let offset = 200;

        // dibagi per chunks menjadi 200 data
        for chunk in self.candidate_ids.chunks(offset) {
            chunk_vacancies.push(chunk);
        }
        chunk_vacancies
    }

    pub fn vacancies_chunk_list(&mut self) -> Vec<&[i32]> {
        let mut chunk_vacancies = Vec::new();

        // defautl offset
        let offset = 200;

        // dibagi per chunks menjadi 200 data
        for chunk in self.vacancies_id.chunks(offset) {
            chunk_vacancies.push(chunk);
        }
        chunk_vacancies
    }
    pub async fn get_last_id(&self) -> Result<Option<u32>, sqlx::Error> {
        let query = sqlx::query(
            "
            select max(vacancy_can_id) from _temp_taekwang_export 
        ",
        )
        .fetch_one(&self.state.db)
        .await?;

        let last_vacancy = query.get(0);

        Ok(last_vacancy)
    }

    pub async fn get_candidate_ids(&mut self, last_vacancy_can_id: &u32) -> Result<(), sqlx::Error> {
        let sql_query = "select DISTINCT(candidate_id) as candidate_id from karirpad_v5.vacancy_apply_candidate where vacancy_can_id > ?";
        let vacancies = sqlx::query_as::<_, VacancyData>(sql_query)
            .bind(last_vacancy_can_id)
            .fetch_all(&self.state.db)
            .await?;
        self.candidate_ids = vacancies.iter().map(|f| f.candidate_id).collect();
        Ok(())
    }

    pub async fn get_vacancy_ids(&mut self, last_vacancy_can_id: &u32) -> Result<(), sqlx::Error> {
        let sql_query = "select vacancy_can_id  from karirpad_v5.vacancy_apply_candidate where vacancy_can_id > ?";
        let vacancies = sqlx::query_as::<_, VacanciesId>(sql_query)
            .bind(last_vacancy_can_id)
            .fetch_all(&self.state.db)
            .await?;
        self.vacancies_id = vacancies.iter().map(|f| f.vacancy_can_id).collect();
        Ok(())
    }

    pub async fn init_data_candidate(&mut self) -> Result<(), sqlx::Error> {
        let mut transaction = self.state.db.begin().await?;
        for chunk in self.vacancies_chunk_list() {
            let query_sql = format!(
                " INSERT INTO `_temp_taekwang_export`  (
                    candidate_id,
                    candidate,
                    phone,
                    whatsapp,
                    email,
                    updated_at,
                    expected_salary,
                    vacancy_id,
                    vacancy,
                    current_salary,
                    applied_date,
                    id_number,
                    employer_id,
                    vacancy_can_id
                )
                SELECT
                    -- Basic candidate information
                    c.candidate_id as candidate_id,
                    c.full_name,
                    c.phone,
                    c.wa_number,
                    c.email,
                    c.updated_at,
                    c.expected_salary,
                    vac.vacancy_id,
                    v.vacancy_name,
                    c.current_salary,
                    vac.apply_date,
                    c.id_number,
                    vac.employer_id,
                    vac.vacancy_can_id
                FROM karirpad_v5.vacancy_apply_candidate vac
                LEFT JOIN karirpad_v5.t_candidate c ON vac.candidate_id = c.candidate_id
                LEFT JOIN karirpad_v5.t_vacancy v ON v.vacancy_id = vac.vacancy_id
                WHERE vac.employer_id = 63402 and vac.vacancy_can_id in ({})
                order by vac.vacancy_can_id asc",
                chunk.iter().map(|f| f.to_string()).collect::<Vec<String>>().join(",")
            );

            let _execute_query = sqlx::query(&query_sql).execute(&mut *transaction).await?;
        }

        transaction.commit().await?;

        Ok(())
    }
    pub async fn first_init_data_reusable(&mut self, sql: &str, label: &str) -> Result<(), sqlx::Error> {
        println!("running  first init {}", label);

        let mut transaction = self.state.db.begin().await?;
        sqlx::query(SEESION_EXPORT).execute(&mut *transaction).await?;
        for chunk in self.chunk_list() {
            let fomratted_chunk = chunk.iter().map(|f| f.to_string()).collect::<Vec<String>>().join(",");

            println!("formatted vacancies_id : {:?}", fomratted_chunk);
            let query_sql = sql.replace("{}", &fomratted_chunk);

            let execute_query = sqlx::query(&query_sql).execute(&mut *transaction).await?;

            println!("{:?}", execute_query);
        }

        transaction.commit().await?;

        println!("success first init {}", label);

        Ok(())
    }

    pub async fn run_worker(&mut self) -> Result<(), sqlx::Error> {
        let last_id = self.get_last_id().await.unwrap().unwrap_or(0);
        self.get_candidate_ids(&last_id).await?;
        self.get_vacancy_ids(&last_id).await?;

        println!("init data candidate");
        self.init_data_candidate().await?;
        println!("success init data candidate");

        let education_query = "UPDATE `_temp_taekwang_export` e
                            JOIN (
                                SELECT
                                    ce.candidate_id,
                                    GROUP_CONCAT(
                                        CONCAT_WS('|',
                                            COALESCE(NULLIF(me.educ_desc_en, ''), ''),
                                            COALESCE(NULLIF(ce.institution_name, ''), ''),
                                            COALESCE(NULLIF(ce.major, ''), ''),
                                            COALESCE(NULLIF(ce.start_year, ''), ''),
                                            COALESCE(NULLIF(ce.end_year, ''), ''),
                                            COALESCE(NULLIF(ce.gpa, ''), ''),
                                            COALESCE(NULLIF(ce.until_now, ''), '')
                                        ) ORDER BY ce.start_year DESC SEPARATOR ';;'
                                    ) AS education_history
                                FROM karirpad_v5.candidate_educ ce
                                 LEFT JOIN karirpad_v5.m_education me ON me.educ_id = ce.educ_id
                                where ce.candidate_id IN ({})
                                GROUP BY ce.candidate_id
                            ) AS ed ON ed.candidate_id = e.candidate_id
                            SET e.education_history = ed.education_history;";

        let language_query = " update `_temp_taekwang_export` e
                        JOIN (
                               SELECT
                        kl.candidate_id,
                        GROUP_CONCAT(
                            CONCAT_WS('|',
                                COALESCE(NULLIF(kl.language_desc, ''), ''),
                                COALESCE(NULLIF(kl.language_test_type, ''), ''),
                                COALESCE(NULLIF(kl.language_score, ''), ''),
                                COALESCE(NULLIF(kl.language_start_date, null), null),
                                COALESCE(NULLIF(kl.language_end_date, null), null)
                            ) SEPARATOR ';;'
                        ) AS language_skills
                                            FROM (
                            SELECT 
                                cl.candidate_id,
                                cl.language_id AS cl_language_id,
                                l.language_desc,
                                cl.language_test_type,
                                cl.language_score,
                                cl.language_start_date,
                                cl.language_end_date
                            FROM karirpad_v5.candidate_language cl
                            LEFT JOIN karirpad_v5.m_language l 
                                ON l.language_id = cl.language_id
                             where cl.candidate_id IN ({})
                            GROUP BY cl.candidate_id, cl.language_id
                            ) kl group by kl.candidate_id
                        ) AS lang ON lang.candidate_id = e.candidate_id
                        SET e.language_skills = lang.language_skills;";

        let family_info = " UPDATE `_temp_taekwang_export` e
                JOIN (
                    SELECT
                        cfd.candidate_id,
                        GROUP_CONCAT(
                            CONCAT_WS('|',
                                COALESCE(NULLIF(cfd.family_nik, ''), ''),
                                COALESCE(NULLIF(cfd.name, ''), ''),
                                COALESCE(NULLIF(rel.relationship_desc, ''), ''),
                                COALESCE(NULLIF(cfd.dob, ''), ''),
                                COALESCE(NULLIF(me.educ_desc_en, ''), ''),
                                COALESCE(NULLIF(cfd.occupation, ''), '')
                            )
                            SEPARATOR ';;'
                        ) AS family_info
                    FROM karirpad_v5.candidate_family_detail cfd
                    LEFT JOIN karirpad_v5.m_relationship rel ON cfd.relationship_id = rel.relationship_id
                    LEFT JOIN karirpad_v5.m_education me ON cfd.educ_id = me.educ_id
                    where cfd.candidate_id IN ({})
                    GROUP BY cfd.candidate_id
                ) fam ON fam.candidate_id = e.candidate_id
                SET e.family_info = fam.family_info;";

        let emergency_contact_query = " UPDATE `_temp_taekwang_export` e
                        JOIN (
                            SELECT
                                ec.candidate_id,
                                GROUP_CONCAT(
                                    CONCAT_WS('|',
                                        COALESCE(NULLIF(ec.name, ''), ''),
                                        COALESCE(NULLIF(rel.relationship_desc, ''), ''),
                                        COALESCE(NULLIF(ec.phone_type, ''), ''),
                                        COALESCE(NULLIF(ec.phone_number, ''), ''),
                                        COALESCE(NULLIF(ec.is_living_together, ''))
                                    ) SEPARATOR ';;'
                                ) AS emergency_contacts
                            from karirpad_v5.candidate_contact ec
                             LEFT JOIN  karirpad_v5.m_relationship rel ON ec.relationship_id = rel.relationship_id
                             LEFT JOIN karirpad_v5.vacancy_apply_candidate vac on vac.candidate_id = ec.candidate_id 
                             where ec.candidate_id IN ({})
                            group by ec.candidate_id
                        ) ec_all ON ec_all.candidate_id = e.candidate_id
                        SET e.emergency_contacts = ec_all.emergency_contacts;";
        let certificates_query = "update `_temp_taekwang_export` tte 
                        join(
                            SELECT
                                    ti.candidate_id,
                                    CONCAT_WS('|',
                                        COALESCE(ml.license_desc, ''),
                                        COALESCE(ti.driving_license_number, '')
                                    ) AS license_info
                                FROM karirpad_v5.t_candidate_info ti
                                LEFT JOIN karirpad_v5.m_license ml ON ml.license_id = ti.driving_license
                                where ti.candidate_id IN ({})
                                group by ti.candidate_id
                        )as dv on dv.candidate_id =  tte.candidate_id
                        SET tte.license_info = dv.license_info;";
        let license_query = "  UPDATE `_temp_taekwang_export` e
                        JOIN (
                            SELECT
                                cc.candidate_id,
                                GROUP_CONCAT(
                                    CONCAT_WS('|',
                                        COALESCE(NULLIF(cc.certificate_number, ''), ''),
                                        COALESCE(NULLIF(cc.topic, ''), ''),
                                        COALESCE(NULLIF(cc.certificate_grade, ''), ''),
                                        COALESCE(NULLIF(cc.organized_by, ''), ''),
                                        COALESCE(NULLIF(cc.start_year, ''), ''),
                                        COALESCE(NULLIF(cc.end_year, ''), '')
                                    )
                                    ORDER BY cc.start_year DESC
                                    SEPARATOR ';;'
                                ) AS certifications
                            FROM karirpad_v5.candidate_course cc
                            WHERE cc.candidate_id IN ({})
                            GROUP BY cc.candidate_id
                        ) certs ON certs.candidate_id = e.candidate_id
                        SET e.certifications = certs.certifications;";

        let work_experience_query = "UPDATE `_temp_taekwang_export` e
                    JOIN (
                        SELECT 
                            cexp.candidate_id,
                            GROUP_CONCAT(
                                CONCAT_WS('|',
                                    COALESCE(NULLIF(cexp.company_name, ''), ''),
                                    COALESCE(NULLIF(cexp.position, ''), ''),
                                    COALESCE(NULLIF(ci.industry_desc, ''), ''),
                                    COALESCE(NULLIF(mjl.job_level_desc, ''), ''),
                                    COALESCE(NULLIF(cexp.start_year, ''), ''),
                                    COALESCE(NULLIF(cexp.end_year, ''), ''),
                                    COALESCE(NULLIF(cexp.reason_for_leaving, ''), ''),
                                    COALESCE(NULLIF(cexp.until_now, ''), '')
                                ) ORDER BY cexp.start_year DESC SEPARATOR ';;'
                            ) AS work_experience
                        FROM karirpad_v5.candidate_exp cexp
                        LEFT JOIN karirpad_v5.mika_m_industry ci ON CAST(ci.industry_code AS CHAR) = cexp.business_line
                        LEFT JOIN karirpad_v5.m_job_level mjl ON mjl.job_level_id = cexp.job_level
                        where cexp.candidate_id IN ({})
                        GROUP BY cexp.candidate_id
                    ) AS ed ON ed.candidate_id = e.candidate_id
                    SET e.work_experience = ed.work_experience;";

        self.first_init_data_reusable(education_query, "Education").await?;
        self.first_init_data_reusable(language_query, "Language").await?;
        self.first_init_data_reusable(family_info, "Family Info").await?;
        self.first_init_data_reusable(emergency_contact_query, "Emergency Contact")
            .await?;
        self.first_init_data_reusable(certificates_query, "Certificates")
            .await?;
        self.first_init_data_reusable(license_query, "License ").await?;
        self.first_init_data_reusable(work_experience_query, "Work Experience")
            .await?;

        Ok(())
    }
}
