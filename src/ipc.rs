use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::ProblemVerdict;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct JudeRequest {
    pub cmd: String,
    pub cmd_options: Option<Vec<String>>,
    pub time: u64,
    pub memory: u64,
    pub test_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JudgeResponse {
    pub ok: bool,
    pub error: Option<String>,
    pub verdict: Option<ProblemVerdict>,
    pub test_case: Option<u32>,
}

impl JudgeResponse {
    pub fn error(err: &str) -> Self {
        Self {
            ok: false,
            error: Some(String::from(err)),
            verdict: None,
            test_case: None,
        }
    }

    pub fn test_case(test_case: u32) -> Self {
        Self {
            ok: true,
            error: None,
            verdict: None,
            test_case: Some(test_case),
        }
    }

    pub fn ok(verdict: ProblemVerdict) -> Self {
        Self {
            ok: true,
            error: None,
            verdict: Some(verdict),
            test_case: None,
        }
    }
}
