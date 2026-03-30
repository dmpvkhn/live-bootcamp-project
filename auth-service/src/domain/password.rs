use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};
use color_eyre::eyre::{eyre, Context, Result};
use secrecy::{ExposeSecret, SecretString};

#[derive(Debug, Clone)]
pub struct HashedPassword(SecretString);

impl PartialEq for HashedPassword {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl HashedPassword {
    pub async fn parse(s: SecretString) -> Result<Self> {
        if !validate_password(&s) {
            return Err(eyre!("Password is too short or empty"));
        }
        let hash = compute_password_hash(&s)
            .await
            .wrap_err("failed to compute password hash")?;
        Ok(HashedPassword(hash))
    }

    pub fn parse_password_hash(hash: SecretString) -> Result<HashedPassword> {
        PasswordHash::new(hash.expose_secret()).map_err(|e| eyre!(e.to_string()))?;
        Ok(HashedPassword(hash))
    }

    #[tracing::instrument(name = "Verify raw password", skip_all)]
    pub async fn verify_raw_password(&self, password_candidate: &SecretString) -> Result<()> {
        let current_span: tracing::Span = tracing::Span::current();
        let password_hash = self.0.expose_secret().to_owned();
        let password_candidate = password_candidate.expose_secret().to_owned();
        tokio::task::spawn_blocking(move || {
            current_span.in_scope(|| {
                let expected_password_hash =
                    PasswordHash::new(&password_hash).wrap_err("failed to parse password hash")?;
                Argon2::default()
                    .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                    .wrap_err("failed to verify password")
            })
        })
        .await?
    }
}

impl AsRef<SecretString> for HashedPassword {
    fn as_ref(&self) -> &SecretString {
        &self.0
    }
}

fn validate_password(s: &SecretString) -> bool {
    s.expose_secret().len() >= 8
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: &SecretString) -> Result<SecretString> {
    let current_span: tracing::Span = tracing::Span::current();
    let password = password.expose_secret().to_owned();
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
            Ok(SecretString::new(hash.into_boxed_str()))
        })
    })
    .await?
}
