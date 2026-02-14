use std::collections::HashSet;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashmapBannedTokenStore {
    pub tokens: HashSet<String>,
}
#[async_trait::async_trait]
impl BannedTokenStore for HashmapBannedTokenStore {
    async fn store_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if !self.tokens.insert(token) {
            Err(BannedTokenStoreError::UnexpectedError)
        } else {
            Ok(())
        }
    }
    async fn is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}
