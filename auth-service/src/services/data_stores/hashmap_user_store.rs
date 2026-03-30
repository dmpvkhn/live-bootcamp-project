use crate::domain::{Email, User, UserStore, UserStoreError};
use secrecy::SecretString;
use std::collections::HashMap;

// TODO: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap`` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.

#[derive(Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(u) => Ok(u.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        raw_password: &SecretString,
    ) -> Result<(), UserStoreError> {
        let user = self.users.get(email).ok_or(UserStoreError::UserNotFound)?;
        user.password
            .verify_raw_password(raw_password)
            .await
            .map_err(|_| UserStoreError::InvalidCredentials)
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use crate::domain::Email;
    use secrecy::SecretString;

    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut storage = HashmapUserStore::default();

        let user = User {
            email: Email::parse(SecretString::new("admin@example.com".into())).unwrap(),
            password: HashedPassword::parse(SecretString::new("password".into()))
                .await
                .unwrap(),
            requires_2fa: true,
        };

        let add_user_result = storage.add_user(user.clone()).await;
        assert_eq!(add_user_result, Ok(()));
        let add_same_user_result = storage.add_user(user.clone()).await;
        assert_eq!(add_same_user_result, Err(UserStoreError::UserAlreadyExists))
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut storage = HashmapUserStore::default();

        let user = User {
            email: Email::parse(SecretString::new("admin@example.com".into())).unwrap(),
            password: HashedPassword::parse("password".to_string()).await.unwrap(),
            requires_2fa: true,
        };

        let _ = storage.add_user(user.clone()).await;

        let get_user_reslt = storage.get_user(&user.email).await;
        assert_eq!(get_user_reslt.is_ok(), true);
        let get_unexist_user_reslt = storage
            .get_user(Email::parse("hacker@example.com".to_string()).unwrap())
            .await;
        assert_eq!(get_unexist_user_reslt, Err(UserStoreError::UserNotFound))
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut storage = HashmapUserStore::default();

        let user = User {
            email: Email::parse("admin@example.com".to_string()).unwrap(),
            password: HashedPassword::parse("password".to_string()).await.unwrap(),
            requires_2fa: true,
        };

        let _ = storage.add_user(user.clone()).await;

        // User not found
        let not_found = storage
            .validate_user(
                &Email::parse(SecretString::new("...".into())).unwrap(),
                &SecretString::new("password".into()),
            )
            .await;

        assert_eq!(not_found, Err(UserStoreError::UserNotFound));
        // Password is wrong
        let wrong_password = storage
            .validate_user(
                Email::parse("admin@example.com".to_string()).unwrap(),
                "password1",
            )
            .await;
        assert_eq!(wrong_password, Err(UserStoreError::InvalidCredentials));
        // everything is correct
        let correct = storage
            .validate_user(
                Email::parse("admin@example.com".to_string()).unwrap(),
                "password",
            )
            .await;
        assert_eq!(correct, Ok(()));
    }
}
