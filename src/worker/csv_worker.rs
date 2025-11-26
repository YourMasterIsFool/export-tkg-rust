use std::{collections::HashMap, fs};

use csv::Writer;

pub fn save_csv_worker(path: &str, filename: &str, records: &Vec<HashMap<String, String>>) -> csv::Result<()> {
    let csv_dir = format!("csv/{}", &path);
    fs::create_dir_all(&csv_dir)?;

    let mut writer = Writer::from_path(format!("{}/{}.csv", csv_dir, filename))?;
    //create path folder

    if let Some(_first) = records.first() {
        let headers = [
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
            "Vacancy",
            "Applied Date",
            "Education 1 - Grades",
            "Education 1 - Institution Name",
            "Education 1 - Major",
            "Education 1 - Start Date",
            "Education 1 - End Date",
            "Education 1 - GPA",
            "Education 1 - Now?",
            "Education 2 - Grades",
            "Education 2 - Institution Name",
            "Education 2 - Major",
            "Education 2 - Start Date",
            "Education 2 - End Date",
            "Education 2 - GPA",
            "Education 2 - Now?",
            "Education 3 - Grades",
            "Education 3 - Institution Name",
            "Education 3 - Major",
            "Education 3 - Start Date",
            "Education 3 - End Date",
            "Education 3 - GPA",
            "Education 3 - Now?",
            "Education 4 - Grades",
            "Education 4 - Institution Name",
            "Education 4 - Major",
            "Education 4 - Start Date",
            "Education 4 - End Date",
            "Education 4 - GPA",
            "Education 4 - Now?",
            "Experience 1 - Company Name",
            "Experience 1 - Position",
            "Experience 1 - Business Line (Industry)",
            "Experience 1 - Job Level",
            "Experience 1 - Start Date",
            "Experience 1 - End Date",
            "Experience 1 - Currently Working Here (Y/N)",
            "Experience 2 - Company Name",
            "Experience 2 - Position",
            "Experience 2 - Business Line (Industry)",
            "Experience 2 - Job Level",
            "Experience 2 - Start Date",
            "Experience 2 - End Date",
            "Experience 2 - Currently Working Here (Y/N)",
            "Experience 3 - Company Name",
            "Experience 3 - Position",
            "Experience 3 - Business Line (Industry)",
            "Experience 3 - Job Level",
            "Experience 3 - Start Date",
            "Experience 3 - End Date",
            "Experience 3 - Currently Working Here (Y/N)",
            "Experience 4 - Company Name",
            "Experience 4 - Position",
            "Experience 4 - Business Line (Industry)",
            "Experience 4 - Job Level",
            "Experience 4 - Start Date",
            "Experience 4 - End Date",
            "Experience 4 - Currently Working Here (Y/N)",
            "Language 1 - Language",
            "Language 1 - Test Type",
            "Language 1 - Score",
            "Language 1 - Acquire Date",
            "Language 1 - Expired Date",
            "Language 2 - Language",
            "Language 2 - Test Type",
            "Language 2 - Score",
            "Language 2 - Acquire Date",
            "Language 2 - Expired Date",
            "Language 3 - Language",
            "Language 3 - Test Type",
            "Language 3 - Score",
            "Language 3 - Acquire Date",
            "Language 3 - Expired Date",
            "Language 4 - Language",
            "Language 4 - Test Type",
            "Language 4 - Score",
            "Language 4 - Acquire Date",
            "Language 4 - Expired Date",
            "Certification 1 - Certificate Number",
            "Certification 1 - Certificate Name",
            "Certification 1 - Grade",
            "Certification 1 - Institution Name",
            "Certification 1 - Acquire Date",
            "Certification 1 - Expired Date",
            "Certification 2 - Certificate Number",
            "Certification 2 - Certificate Name",
            "Certification 2 - Grade",
            "Certification 2 - Institution Name",
            "Certification 2 - Acquire Date",
            "Certification 2 - Expired Date",
            "Certification 3 - Certificate Number",
            "Certification 3 - Certificate Name",
            "Certification 3 - Grade",
            "Certification 3 - Institution Name",
            "Certification 3 - Acquire Date",
            "Certification 3 - Expired Date",
            "Certification 4 - Certificate Number",
            "Certification 4 - Certificate Name",
            "Certification 4 - Grade",
            "Certification 4 - Institution Name",
            "Certification 4 - Acquire Date",
            "Certification 4 - Expired Date",
        ];
        writer.write_record(headers)?; // ✅ Added & for write_record

        for row in records {
            let record: Vec<String> = headers
                .iter()
                .map(|field| row.get(&field.to_string()).cloned().unwrap_or_else(|| "".to_string()))
                .collect();

            writer.write_record(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}
