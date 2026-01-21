use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyTokenError {
    pub error: String,
}
