use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};
use color_eyre::eyre::{Context, Result};
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub async fn parse(s: String) -> Result<Self> {
        if s.len() < 8 {
            return Err(color_eyre::eyre::eyre!("Password is too short or empty"));
        }

        let hash = compute_password_hash(&s)
            .await
            .wrap_err("failed to compute password hash")?;

        Ok(HashedPassword(hash))
    }

    pub fn parse_password_hash(hash: String) -> Result<HashedPassword, String> {
        PasswordHash::new(&hash).map_err(|e| e.to_string())?;
        Ok(HashedPassword(hash))
    }

    #[tracing::instrument(name = "Verify raw password", skip_all)]
    pub async fn verify_raw_password(
        &self,
        password_candidate: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let current_span: tracing::Span = tracing::Span::current();
        let password_hash = self.as_ref().to_owned();
        let password_candidate = password_candidate.to_owned();
        tokio::task::spawn_blocking(move || {
            current_span.in_scope(|| {
                let expected_password_hash = PasswordHash::new(&password_hash)?;
                Argon2::default()
                    .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                    .map_err(|e| e.into())
            })
        })
        .await?
    }
}

impl AsRef<str> for HashedPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: &str) -> Result<String> {
    let current_span: tracing::Span = tracing::Span::current();
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt = SaltString::generate(&mut OsRng);
            let hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
            Ok(hash)
        })
    })
    .await?
}
