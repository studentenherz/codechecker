use std::error::Error;
use std::io::BufReader;
use std::io::Write;
use std::process::Command;

mod checker;
mod process;
mod utils;

pub use checker::*;
use process::*;
use tqdm::{Iter, Style};
use utils::*;

#[derive(Debug)]
pub enum ProblemVerdict {
    Accepted { time: u64, memory: u64 },
    WrongAnswer { msg: String },
    TimeLimitExceeded,
    MemoryLimitExceeded,
    IdleLimitExceeded,
    RuntimeError(i32),
}

/// Judge a problem against a single test case
///
/// # Arguments
///
/// `executable_path`: Path to the executable
/// `input_path`: Path to the input file
/// `time_limit`: Time limit in ms
/// `memory_limit`: Memory limit in Mb
/// `checker`: The checker that checks for correctness
///
/// # Returns
///
/// The verdict of the judge
pub fn judge(
    executable_path: &str,
    input_path: &str,
    time_limit: u64,
    memory_limit: u64,
    checker: impl Checker,
) -> Result<ProblemVerdict, Box<dyn Error>> {
    let input = std::fs::read_to_string(input_path)?;

    let mut child = Command::new(executable_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let pid = Pid::from_raw(child.id() as i32);
    let mut process = Process::new(pid, time_limit, memory_limit);

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    wait_for(&mut process);

    match process.state {
        ProcessState::Exited(0) => {
            let stdout = child.stdout.as_mut().unwrap();
            let mut reader = BufReader::new(stdout);

            match checker.check(&mut reader) {
                Ok(()) => Ok(ProblemVerdict::Accepted {
                    time: process.consumed_time_ms,
                    memory: process.consumed_memory_mb,
                }),
                Err(msg) => Ok(ProblemVerdict::WrongAnswer { msg }),
            }
        }
        ProcessState::Exited(_) => Ok(ProblemVerdict::RuntimeError(0)),
        ProcessState::TimeLimitExceeded => Ok(ProblemVerdict::TimeLimitExceeded),
        ProcessState::MemoryLimitExceeded => Ok(ProblemVerdict::MemoryLimitExceeded),
        ProcessState::IdleLimitExceeded => Ok(ProblemVerdict::IdleLimitExceeded),
        ProcessState::RuntimeError(sig) => Ok(ProblemVerdict::RuntimeError(sig)),
        _ => Err("An unexpected error ocurred".into()),
    }
}

/// Judge a problem against a set of test cases
///
/// # Arguments
///
/// `executable_path`: Path to the executable
/// `directory`: Directory where the testcases live
/// `time_limit`: Time limit in ms
/// `memory_limit`: Memory limit in Mb
///
/// # Returns
///
/// The verdict of the judge along with the last test case it failed
/// (in case it did fail).
/// If it's Accepted it returns the maximum time and memoy usage.
pub fn judge_all(
    executable_path: &str,
    directory: &str,
    time_limit: u64,
    memory_limit: u64,
) -> Result<(ProblemVerdict, u32), Box<dyn Error>> {
    let numbers = sorted_list_numbers_in_folder(directory)?;

    let mut max_time: u64 = 0;
    let mut max_memory: u64 = 0;

    for num in numbers
        .into_iter()
        .tqdm()
        .desc(Some("Testing..."))
        .width(Some(100))
        .style(Style::Balloon)
    {
        let input = format!("{}/{}.in", directory, num);
        let output = format!("{}/{}.out", directory, num);

        let checker = LinesChecker::new(&output);

        match judge(executable_path, &input, time_limit, memory_limit, checker) {
            Ok(ProblemVerdict::Accepted { time, memory }) => {
                max_time = std::cmp::max(max_time, time);
                max_memory = std::cmp::max(max_memory, memory);
            }
            Ok(err_verdict) => return Ok((err_verdict, num)),
            Err(err) => return Err(err),
        }
    }

    Ok((
        ProblemVerdict::Accepted {
            time: max_time,
            memory: max_memory,
        },
        0,
    ))
}
