use crate::domain::HashedPassword;
use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, SecretString};
use std::hash::Hash;
pub use validator::Validate;
pub use validator::ValidationError;

pub enum EmailError {
    InvalidEmail,
}

#[derive(Validate)]
struct EmailValidator {
    #[validate(email)]
    email: String,
}

#[derive(Debug, Clone)]
pub struct Email(SecretString);

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Eq for Email {}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl AsRef<SecretString> for Email {
    fn as_ref(&self) -> &SecretString {
        &self.0
    }
}

impl Email {
    pub fn parse(email: SecretString) -> Result<Self> {
        let validator = EmailValidator {
            email: email.expose_secret().to_string(),
        };

        // Validate using the validator crate
        validator.validate().map_err(|_| eyre!("Invalid email"))?;
        Ok(Email(email))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: HashedPassword,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: Email, password: HashedPassword, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }
}
