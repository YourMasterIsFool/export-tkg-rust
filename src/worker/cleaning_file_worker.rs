use std::fs;

use anyhow::Result;

pub fn cleaning_file(csv_path: &str, excel_path: &str) -> Result<()> {
    let path_csv = format!("csv/{}", csv_path);
    let _excel_path_file = "output.xlsx".to_string();

    if fs::remove_dir_all(path_csv).is_err() {
        println!("failed remove path");
    }

    if fs::remove_file(excel_path).is_err() {
        println!("failed remove excel file");
    }

    Ok(())
}
