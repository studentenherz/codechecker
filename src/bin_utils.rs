use std::error::Error;
use std::io::BufReader;
use std::io::Write;
use std::process::Command;

pub use crate::checker::*;
use crate::process::*;
pub use crate::utils::*;

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
