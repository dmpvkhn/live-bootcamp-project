use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUPRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUPResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUPError {
    pub error: String,
}
