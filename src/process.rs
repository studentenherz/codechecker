use libc::rusage;
use libc::wait4;
use nix::sys::signal::kill;
use nix::sys::signal::Signal;
use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub use nix::unistd::Pid;

#[derive(Debug)]
pub enum ProcessState {
    Running,
    Exited(i32),
    TimeLimitExceeded,
    MemoryLimitExceeded,
    IdleLimitExceeded,
    RuntimeError(i32),
    Failed,
}

#[derive(Debug)]
pub struct Process {
    pid: Pid,
    time_limit_ms: u64,
    memory_limit_mb: u64,
    pub consumed_time_ms: u64,
    pub consumed_memory_mb: u64,
    idle_count: u32,
    pub state: ProcessState,
}

impl Process {
    pub fn new(pid: Pid, time_limit_ms: u64, memory_limit_mb: u64) -> Self {
        Process {
            pid,
            time_limit_ms,
            memory_limit_mb,
            consumed_time_ms: 0,
            consumed_memory_mb: 0,
            idle_count: 0,
            state: ProcessState::Running,
        }
    }
}

/// Convert clock ticks to milliseconds
fn ticks_to_ms(ticks: u64) -> u64 {
    let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as u64;
    ticks * 1000 / ticks_per_sec
}

fn read_stat(pid: Pid) -> u64 {
    let path = format!("/proc/{}/stat", pid);
    let file = File::open(path).expect("Failed to open stat file");
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    let values: Vec<&str> = line.split_whitespace().collect();
    let utime: u64 = values[13].parse().unwrap();
    let stime: u64 = values[14].parse().unwrap();

    ticks_to_ms(utime + stime)
}

fn read_memory_usage(pid: Pid) -> u64 {
    let path = format!("/proc/{}/status", pid);
    let file = File::open(path).expect("Failed to open status file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("VmPeak:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let value_kb: u64 = parts[1].parse().unwrap();
            return value_kb / 1024; // Return un Mb
        }
    }

    0
}

/// Update process with /proc/{pid}/ files
fn update(process: &mut Process) {
    let cpu_time = read_stat(process.pid);
    let memory = read_memory_usage(process.pid);

    if process.consumed_time_ms == cpu_time {
        process.idle_count += 1;
    } else {
        process.idle_count = 0;
    }

    process.consumed_time_ms = max(process.consumed_time_ms, cpu_time);
    process.consumed_memory_mb = max(process.consumed_memory_mb, memory);
}

/// Update process with resource usage via `rusage` from `wait4`
fn update_with_rusage(process: &mut Process, usage: &rusage) {
    let total_time = usage.ru_utime.tv_sec as u64 * 1000
        + usage.ru_utime.tv_usec as u64 / 1000
        + usage.ru_stime.tv_sec as u64 * 1000
        + usage.ru_stime.tv_usec as u64 / 1000;

    process.consumed_time_ms = max(process.consumed_time_ms, total_time);
    process.consumed_memory_mb = max(process.consumed_memory_mb, usage.ru_maxrss as u64 / 1024);
}

fn is_idle(process: &Process, elapsed_time: u64) -> bool {
    process.idle_count > 100 || (elapsed_time > 5000 && elapsed_time > 10 * process.time_limit_ms)
}

pub fn wait_for(process: &mut Process) {
    let start_time = SystemTime::now();
    let mut iter = 0;

    while matches!(process.state, ProcessState::Running) {
        iter += 1;

        let mut status: i32 = 0;
        let mut usage: rusage = unsafe { std::mem::zeroed() };

        let wait_result = unsafe {
            wait4(
                process.pid.into(),
                &mut status,
                libc::WCONTINUED | libc::WUNTRACED | libc::WNOHANG,
                &mut usage,
            )
        };

        if wait_result == 0 {
            update(process);

            if process.consumed_time_ms > process.time_limit_ms && process.time_limit_ms > 0 {
                process.state = ProcessState::TimeLimitExceeded;
                if kill(process.pid, Signal::SIGKILL).is_err() {
                    process.state = ProcessState::Failed;
                }
                return;
            }

            if process.consumed_memory_mb > process.memory_limit_mb && process.memory_limit_mb > 0 {
                process.state = ProcessState::MemoryLimitExceeded;
                if kill(process.pid, Signal::SIGKILL).is_err() {
                    process.state = ProcessState::Failed;
                }
                return;
            }

            let elapsed_time = start_time.elapsed().unwrap().as_millis() as u64;
            if is_idle(process, elapsed_time) {
                process.state = ProcessState::IdleLimitExceeded;
                if kill(process.pid, Signal::SIGKILL).is_err() {
                    process.state = ProcessState::Failed;
                }
                return;
            }

            sleep(Duration::from_millis(min(100 * iter, 1000) as u64));
        } else {
            update_with_rusage(process, &usage);

            if process.consumed_time_ms > process.time_limit_ms && process.time_limit_ms > 0 {
                process.state = ProcessState::TimeLimitExceeded;
                if kill(process.pid, Signal::SIGKILL).is_err() {
                    process.state = ProcessState::Failed;
                }
                return;
            }

            if process.consumed_memory_mb > process.memory_limit_mb && process.memory_limit_mb > 0 {
                process.state = ProcessState::MemoryLimitExceeded;
                if kill(process.pid, Signal::SIGKILL).is_err() {
                    process.state = ProcessState::Failed;
                }
                return;
            }

            if wait_result == -1 {
                process.state = ProcessState::Failed;
                return;
            }

            if wait_result == process.pid.into() && libc::WIFSTOPPED(status) {
                process.state = ProcessState::RuntimeError(libc::WSTOPSIG(status));
                return;
            }

            if wait_result == process.pid.into() && libc::WIFSIGNALED(status) {
                process.state = ProcessState::RuntimeError(libc::WTERMSIG(status));
                return;
            }

            if wait_result == process.pid.into() && libc::WIFEXITED(status) {
                process.state = ProcessState::Exited(libc::WEXITSTATUS(status));
                return;
            }

            println!(
                "Unexpected process state: wait4_result={}, status={}",
                wait_result, status
            );
            process.state = ProcessState::Failed;
        }
    }
}
