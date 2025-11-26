use std::collections::HashMap;

pub fn formatted_data(records: HashMap<String, String>) -> HashMap<String, String> {
    let mut column_groups = HashMap::<&str, Vec<&str>>::new();

    // insert basic info
    column_groups.insert(
        "basic_info",
        vec![
            "Candidate ID",
            "Candidate",
            "NIK KTP",
            "Full Name",
            "Phone",
            "Phone (Whatsapp)",
            "Email",
            "Driving License",
            "Driving License Number",
            "Expected Salary",
        ],
    );

    column_groups.insert("vacancy_applied", vec!["Vacancy", "Applied Date"]);

    column_groups.insert(
        "education",
        vec![
            "Education {} - Grades",
            "Education {} - Institution Name",
            "Education {} - Major",
            "Education {} - Start Date",
            "Education {} - End Date",
            "Education {} - GPA",
            "Education {} - Now?",
        ],
    );

    column_groups.insert(
        "experience",
        vec![
            "Experience {} - Company Name",
            "Experience {} - Position",
            "Experience {} - Business Line (Industry)",
            "Experience {} - Job Level",
            "Experience {} - Start Date",
            "Experience {} - End Date",
            "Experience {} - Currently Working Here (Y/N)",
        ],
    );

    column_groups.insert(
        "language",
        vec![
            "Language {} - Language",
            "Language {} - Test Type",
            "Language {} - Score",
            "Language {} - Acquire Date",
            "Language {} - Expired Date",
        ],
    );

    column_groups.insert(
        "certification",
        vec![
            "Certification {} - Certificate Number",
            "Certification {} - Certificate Name",
            "Certification {} - Grade",
            "Certification {} - Institution Name",
            "Certification {} - Acquire Date",
            "Certification {} - Expired Date",
        ],
    );

    let mut format_data: HashMap<String, String> = HashMap::<String, String>::new();

    for (key, value) in &column_groups {
        if *key == "basic_info" {
            for field in value {
                if *field == "Candidate ID" {
                    let data = if let Some(value) = records.get("candidate_id") {
                        value
                    } else {
                        "_"
                    };

                    format_data.insert(field.to_string(), data.to_string());
                } else if ["Candidate", "Fullname"].contains(field) {
                    let data = if let Some(value) = records.get("full_name") {
                        value
                    } else {
                        "_"
                    };
                    format_data.insert(field.to_string(), data.to_string());
                } else if *field == "NIK KTP" {
                    let data = if let Some(value) = records.get("id_number") {
                        value
                    } else {
                        "_"
                    };
                    format_data.insert(field.to_string(), data.to_string());
                } else if *field == "Phone" {
                    let data = if let Some(value) = records.get("phone") {
                        value
                    } else {
                        "_"
                    };
                    format_data.insert(field.to_string(), data.to_string());
                } else if *field == "Phone (Whatsapp)" {
                    let data = if let Some(value) = records.get("wa_number") {
                        value
                    } else {
                        "_"
                    };
                    format_data.insert(field.to_string(), data.to_string());
                } else if *field == "Email" {
                    let data = if let Some(value) = records.get("email") {
                        value
                    } else {
                        "_"
                    };
                    format_data.insert(field.to_string(), data.to_string());
                } else if ["Driving License", "Driving License Number"].contains(field) {
                    let license_data = if let Some(data) = records.get("license_info") {
                        data.split("|").collect::<Vec<&str>>()
                    } else {
                        vec!["-", "-"]
                    };

                    let driving_license = license_data.first().unwrap_or(&"-");
                    let driving_license_number = license_data.get(1).unwrap_or(&"-");

                    format_data
                        .insert(String::from("Driving License"), driving_license.to_string());
                    format_data.insert(
                        String::from("Driving License Number"),
                        driving_license_number.to_string(),
                    );
                }
            }
        } else if *key == "vacancy_applied" {
            for field in value {
                if *field == "Vacancy" {
                    let data = if let Some(value) = records.get("vacancy") {
                        value
                    } else {
                        "_"
                    };

                    format_data.insert(field.to_string(), data.to_string());
                } else if *field == "Applied Date" {
                    let data = if let Some(value) = records.get("applied_date") {
                        value
                    } else {
                        "_"
                    };

                    format_data.insert(field.to_string(), data.to_string());
                }
            }
        } else if *key == "certification" {
            formatted_entries(
                &mut format_data,
                &records,
                &column_groups,
                "certifications",
                "certification",
            );
        } else if *key == "experience" {
            // // format certifications
            formatted_entries(
                &mut format_data,
                &records,
                &column_groups,
                "work_experience",
                "experience",
            );
        } else if *key == "language" {
            // // format certifications
            formatted_entries(
                &mut format_data,
                &records,
                &column_groups,
                "language_skills",
                "language",
            );
        } else if *key == "education" {
            formatted_entries(
                &mut format_data,
                &records,
                &column_groups,
                "education_history",
                "education",
            );
        }
    }

    format_data
}

pub fn formatted_entries(
    formatted: &mut HashMap<String, String>,
    records: &HashMap<String, String>,
    templates: &HashMap<&str, Vec<&str>>,
    key: &str,
    key_templates: &str,
) {
    let entries = if let Some(data) = records.get(key) {
        data.split(";;").collect()
    } else {
        Vec::<&str>::new()
    };

    // complexity 0(n^2)
    for i in 0..=4 {
        if let Some(value) = &templates.get(key_templates) {
            for (index, template) in value.iter().enumerate() {
                let len_entries = entries.len();
                let increment = (i + 1).to_string();

                let template_format = template.replace("{}", &increment);
                if index < len_entries {
                    let value = entries.get(i).copied().unwrap_or("-");
                    formatted.insert(template_format, value.to_string());
                } else {
                    formatted.insert(template_format, String::from("-"));
                }
            }
        }
    }
}
