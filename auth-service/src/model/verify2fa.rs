use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyRequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyError {
    pub error: String,
}
