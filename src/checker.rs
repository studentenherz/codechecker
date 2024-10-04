use std::fs::File;
use std::io::{BufRead, BufReader};

pub trait Checker {
    fn check(&self, out_reader: &mut impl BufRead) -> Result<(), String>;
}

pub struct LinesChecker {
    answer_path: String,
}

impl LinesChecker {
    pub fn new(answer_path: &str) -> Self {
        Self {
            answer_path: String::from(answer_path),
        }
    }
}

impl Checker for LinesChecker {
    fn check(&self, out_reader: &mut impl BufRead) -> Result<(), String> {
        let answer_file = File::open(&self.answer_path).unwrap();
        let mut answer_reader = BufReader::new(answer_file);

        let mut ans_buf = String::new();
        let mut out_buf = String::new();

        let mut line = 0;
        loop {
            line += 1;
            ans_buf.clear();
            out_buf.clear();
            let ans_res = answer_reader
                .read_line(&mut ans_buf)
                .expect("Error reading from answer file");
            let out_res = out_reader
                .read_line(&mut out_buf)
                .expect("Error reading from stdout of the program");

            if ans_buf.trim() != out_buf.trim() {
                return Err(format!("Wrong answer in line {}", line));
            }

            if ans_res == 0 && out_res != 0 {
                return Err(format!("Wrong answer in line {}", line));
            }

            if ans_res != 0 && out_res == 0 {
                return Err(format!("Wrong answer in line {}", line));
            }

            if ans_res == 0 && out_res == 0 {
                return Ok(());
            }
        }
    }
}
