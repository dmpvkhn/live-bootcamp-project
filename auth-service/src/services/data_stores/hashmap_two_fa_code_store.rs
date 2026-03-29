use std::collections::HashMap;

use async_trait::async_trait;
use color_eyre::eyre::eyre;

use crate::domain::{
    Email, {LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

// TODO: implement TwoFACodeStore for HashmapTwoFACodeStore
#[async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes
            .remove(email)
            .ok_or(TwoFACodeStoreError::UnexpectedError(eyre!(
                "Error remove code"
            )))?;
        Ok(())
    }
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .cloned()
            .ok_or(TwoFACodeStoreError::UnexpectedError(eyre!(
                "Error get code"
            )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Email, LoginAttemptId, TwoFACode};

    fn email() -> Email {
        Email::parse("test@example.com".to_string()).unwrap()
    }

    #[tokio::test]
    async fn add_code_and_get_code_returns_stored_values() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = email();
        let id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .add_code(email.clone(), id.clone(), code.clone())
            .await
            .unwrap();
        let (stored_id, stored_code) = store.get_code(&email).await.unwrap();

        assert_eq!(stored_id, id);
        assert_eq!(stored_code, code);
    }

    #[tokio::test]
    async fn get_code_returns_error_for_nonexistent_email() {
        let store = HashmapTwoFACodeStore::default();
        let result = store.get_code(&email()).await;

        assert!(matches!(
            result,
            Err(TwoFACodeStoreError::UnexpectedError(_))
        ))
    }

    #[tokio::test]
    async fn remove_code_succeeds_after_add() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = email();

        store
            .add_code(
                email.clone(),
                LoginAttemptId::default(),
                TwoFACode::default(),
            )
            .await
            .unwrap();
        store.remove_code(&email).await.unwrap();

        // After removal, get_code should fail
        let result = store.get_code(&email).await;
        assert!(matches!(
            result,
            Err(TwoFACodeStoreError::UnexpectedError(_))
        ))
    }

    #[tokio::test]
    async fn remove_code_returns_error_for_nonexistent_email() {
        let mut store = HashmapTwoFACodeStore::default();
        let result = store.remove_code(&email()).await;

        assert!(matches!(
            result,
            Err(TwoFACodeStoreError::UnexpectedError(_))
        ))
    }

    #[tokio::test]
    async fn add_code_overwrites_existing_entry() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = email();
        let id1 = LoginAttemptId::default();
        let code1 = TwoFACode::default();
        let id2 = LoginAttemptId::default();
        let code2 = TwoFACode::default();

        store.add_code(email.clone(), id1, code1).await.unwrap();
        store
            .add_code(email.clone(), id2.clone(), code2.clone())
            .await
            .unwrap();

        let (stored_id, stored_code) = store.get_code(&email).await.unwrap();
        assert_eq!(stored_id, id2);
        assert_eq!(stored_code, code2);
    }
}
