use std::process::Command;
use std::sync::Once;

use codechecker::{judge_all, ProblemVerdict};

static COMPILATION_DONE: Once = Once::new();
const CPP_FILES: [&str; 4] = ["accepted.cpp", "tle.cpp", "mle.cpp", "wrong_answer.cpp"];

fn compile_cpp_files() {
    COMPILATION_DONE.call_once(|| {
        for file in CPP_FILES {
            let file = format!("tests/{}", file);
            let output = Command::new("g++")
                .arg("-o")
                .arg(file.replace(".cpp", ".exe"))
                .arg(file)
                .output()
                .expect("failed to compile C++ file");
            assert!(output.status.success());
        }
    });
}

#[test]
fn test_wrong_answer() {
    compile_cpp_files();
    let res = judge_all("tests/wrong_answer.exe", "tests/test_cases/", 1000, 128);

    match res {
        Ok((ProblemVerdict::WrongAnswer { .. }, 1)) => {}
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_mle() {
    compile_cpp_files();
    let res = judge_all("tests/mle.exe", "tests/test_cases/", 1000, 128);

    match res {
        Ok((ProblemVerdict::MemoryLimitExceeded, 3)) => {}
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_tle() {
    compile_cpp_files();
    let res = judge_all("tests/tle.exe", "tests/test_cases/", 1000, 128);

    match res {
        Ok((ProblemVerdict::TimeLimitExceeded, 4)) => {}
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_accepted() {
    compile_cpp_files();
    let res = judge_all("tests/accepted.exe", "tests/test_cases/", 1000, 128);

    match res {
        Ok((ProblemVerdict::Accepted { .. }, 0)) => {}
        _ => panic!("Unexpected result"),
    }
}
