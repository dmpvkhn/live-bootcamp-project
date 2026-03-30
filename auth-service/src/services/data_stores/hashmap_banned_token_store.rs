use crate::domain::{BannedTokenStore, BannedTokenStoreError};
use secrecy::{ExposeSecret, SecretString};

use std::collections::HashSet;

#[derive(Default)]
pub struct HashmapBannedTokenStore {
    pub tokens: HashSet<String>,
}
#[async_trait::async_trait]
impl BannedTokenStore for HashmapBannedTokenStore {
    async fn store_token(&mut self, token: SecretString) -> Result<(), BannedTokenStoreError> {
        self.tokens.insert(token.expose_secret().to_owned());
        Ok(())
    }
    async fn is_banned(&self, token: &SecretString) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token.expose_secret()))
    }
}
