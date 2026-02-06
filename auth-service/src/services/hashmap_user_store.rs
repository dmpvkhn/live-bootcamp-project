use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

// TODO: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap`` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.

#[derive(Default)]
pub struct HashmapUserStore {
    pub users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(u) => Ok(u.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(u) if u.password == password => Ok(()),
            Some(_) => Err(UserStoreError::InvalidCredentials),
            _ => Err(UserStoreError::UserNotFound),
        }
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut storage = HashmapUserStore::default();

        let user = User {
            email: "admin@exmple.com".to_string(),
            password: "password".to_string(),
            requires_2fa: true,
        };

        let add_user_result = storage.add_user(user.clone());
        assert_eq!(add_user_result, Ok(()));
        let add_same_user_result = storage.add_user(user.clone());
        assert_eq!(add_same_user_result, Err(UserStoreError::UserAlreadyExists))
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut storage = HashmapUserStore::default();

        let user = User {
            email: "admin@example.com".to_string(),
            password: "password".to_string(),
            requires_2fa: true,
        };

        let _ = storage.add_user(user.clone());

        let get_user_reslt = storage.get_user("admin@example.com");
        assert_eq!(get_user_reslt.is_ok(), true);
        let get_unexist_user_reslt = storage.get_user("hacker@example.com");
        assert_eq!(get_unexist_user_reslt, Err(UserStoreError::UserNotFound))
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut storage = HashmapUserStore::default();

        let user = User {
            email: "admin@example.com".to_string(),
            password: "password".to_string(),
            requires_2fa: true,
        };

        let _ = storage.add_user(user.clone());

        // User not found
        let not_found = storage.validate_user("hacker@example.com", "password");
        assert_eq!(not_found, Err(UserStoreError::UserNotFound));
        // Password is wrong
        let wrong_password = storage.validate_user("admin@example.com", "password1");
        assert_eq!(wrong_password, Err(UserStoreError::InvalidCredentials));
        // everything is correct
        let correct = storage.validate_user("admin@example.com", "password");
        assert_eq!(correct, Ok(()));
    }
}
