use std::fs;

use anyhow::Result;

pub fn cleaning_file(csv_path: &str, excel_path: &str) -> Result<()> {
    let path_csv = format!("csv/{}", csv_path);
    let excel_path_file = format!("{}", "output.xlsx");

    if !fs::remove_dir_all(path_csv).is_ok() {
        println!("failed remove path");
        panic!("failed remove file csv file path");
    }

    if !fs::remove_file(excel_path).is_ok() {
        println!("failed remove excel file");
        panic!("failed remove excel file path");
    }

    Ok(())
}
