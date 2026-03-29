use crate::domain::HashedPassword;
use color_eyre::eyre::{eyre, Result};
pub use validator::Validate;
pub use validator::ValidationError;

pub enum EmailError {
    InvalidEmail,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Email(String);

#[derive(Validate)]
struct EmailValidator {
    #[validate(email)]
    email: String,
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Email {
    pub fn parse(email: String) -> Result<Self> {
        let validator = EmailValidator {
            email: email.clone(),
        };

        // Validate using the validator crate
        validator.validate().map_err(|_| eyre!("Invalid email"))?;

        Ok(Email(email))
    }
}

// The User struct should contain 3 fields. email, which is a String;
// password, which is also a String; and requires_2fa, which is a boolean.
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
