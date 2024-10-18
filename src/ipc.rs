use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::ProblemVerdict;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct JudeRequest {
    pub cmd: String,
    pub cmd_options: Option<Vec<String>>,
    pub time: Option<u64>,
    pub memory: Option<u64>,
    pub test_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JudgeResponse {
    pub verdict: ProblemVerdict,
    pub test_case: u32,
    pub last_test_case: bool,
}
