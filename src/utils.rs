use serde::Serialize;
use std::fs;

#[derive(Debug, Serialize)]
pub enum ProblemVerdict {
    Accepted { time: u64, memory: u64 },
    WrongAnswer { msg: String },
    TimeLimitExceeded,
    MemoryLimitExceeded,
    IdleLimitExceeded,
    RuntimeError(i32),
}

pub fn sorted_list_numbers_in_folder(
    folder_path: &str,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let mut numbers = Vec::new();

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            if file_name.ends_with(".in") {
                if let Some(number_str) = file_name.strip_suffix(".in") {
                    if let Ok(number) = number_str.parse::<u32>() {
                        numbers.push(number);
                    }
                }
            }
        }
    }

    numbers.sort_unstable();

    Ok(numbers)
}
