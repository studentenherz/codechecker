use std::process::Command;
use std::sync::Once;

use codechecker::{judge, LinesChecker, ProblemVerdict};

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
    let checker = LinesChecker::new("tests/test_cases/1.out");
    let res = judge(
        "tests/wrong_answer.exe",
        "tests/test_cases/1.in",
        1000,
        128,
        checker,
    );

    match res {
        Ok(ProblemVerdict::WrongAnswer { .. }) => {}
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_mle() {
    compile_cpp_files();
    let checker = LinesChecker::new("tests/test_cases/3.out");
    let res = judge("tests/mle.exe", "tests/test_cases/3.in", 1000, 128, checker);

    match res {
        Ok(ProblemVerdict::MemoryLimitExceeded) => {}
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_tle() {
    compile_cpp_files();
    let checker = LinesChecker::new("tests/test_cases/4.out");
    let res = judge("tests/tle.exe", "tests/test_cases/4.in", 1000, 128, checker);

    match res {
        Ok(ProblemVerdict::TimeLimitExceeded) => {}
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_accepted() {
    compile_cpp_files();
    let checker = LinesChecker::new("tests/test_cases/4.out");
    let res = judge(
        "tests/accepted.exe",
        "tests/test_cases/4.in",
        1000,
        128,
        checker,
    );

    match res {
        Ok(ProblemVerdict::Accepted { .. }) => {}
        _ => panic!("Unexpected result"),
    }
}
