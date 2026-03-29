use crate::domain::{BannedTokenStore, BannedTokenStoreError};
use color_eyre::eyre;
use std::collections::HashSet;

#[derive(Default)]
pub struct HashmapBannedTokenStore {
    pub tokens: HashSet<String>,
}
#[async_trait::async_trait]
impl BannedTokenStore for HashmapBannedTokenStore {
    async fn store_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if !self.tokens.insert(token) {
            Err(BannedTokenStoreError::UnexpectedError(eyre::eyre!(
                "token already exists"
            )))
        } else {
            Ok(())
        }
    }
    async fn is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}
