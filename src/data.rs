use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppResult;

type FileUniqueId = String;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct StickersEntry {
    pub id: Uuid,
    pub file_id: String,
    pub last_usage_time: DateTime<Local>,
}

impl StickersEntry {
    pub fn new(file_id: String) -> Self {
        StickersEntry {
            id: Uuid::new_v4(),
            file_id: file_id,
            last_usage_time: Local::now(),
        }
    }

    pub fn update(&mut self, file_id: String) {
        self.file_id = file_id;
        self.last_usage_time = Local::now();
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub struct Data {
    pub stickers: HashMap<FileUniqueId, StickersEntry>,
}

impl Data {
    pub fn read_from(path: &Path) -> AppResult<Self> {
        if !path.exists() {
            let new_file = File::create(path)?;
            serde_json::to_writer(new_file, &Data::default())?;
        }
        let file = File::open(path)?;
        serde_json::from_reader(file).map_err(|error| error.into())
    }

    pub fn write_to(&self, path: &Path) -> AppResult<()> {
        let file = if path.exists() {
            OpenOptions::new().write(true).open(path)?
        } else {
            File::create(path)?
        };
        serde_json::to_writer(file, self).map_err(|error| error.into())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_file;
    use std::path::Path;

    use super::Data;

    #[test]
    fn it_worked() {
        let test_file = Path::new("test.json");
        if test_file.exists() {
            remove_file(test_file).unwrap();
        }
        let data = Data::read_from(test_file).unwrap();
        assert_eq!(data, Data::default());
        remove_file(test_file).unwrap();
        data.write_to(test_file).unwrap();
        remove_file(test_file).unwrap();
    }
}
