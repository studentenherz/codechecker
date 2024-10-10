use serde::Serialize;

use super::ProblemVerdict;

#[derive(Debug, Serialize)]
pub struct JudeRequest {
    pub cmd: String,
    pub cmd_options: Option<Vec<String>>,
    pub time: Option<u64>,
    pub memory: Option<u64>,
    pub test_dir: String,
}

#[derive(Debug, Serialize)]
pub struct JudgeResponse {
    pub verdict: ProblemVerdict,
    pub test_case: u32,
    pub last_test_case: bool,
}
