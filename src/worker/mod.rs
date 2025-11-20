use std::fs;

use csv::Reader;

pub mod csv_worker;
pub mod excel_worker;
pub mod fetch;
pub mod worker;

pub fn convert_csv_into_excel(csv_folder: &str) {
    match fs::read_dir(&csv_folder) {
        Err(err) => {
            panic!("folder not found {}: {}", csv_folder, err);
        }
        Ok(entries) => {
            for entry in entries.flatten() {
                // println!("{:?}", entry);
                let path = entry.path();
                println!("{:?}", path.display());

                // check is file
                if path.is_file() {
                    // check exension
                    if let Some(ext) = path.extension() {
                        // if csv
                        println!("{:?}", path.display());

                        // if (ext == "csv") {
                        //     println!("{:?}", path.display());
                        //     // let read_csv = Reader::from_path(path)
                        // }
                    }
                }
            }
        }
    }
}
