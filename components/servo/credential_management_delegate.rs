/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::sync::Mutex;

use servo_url::ImmutableOrigin;

pub trait CredentialManagementDelegate {
    /// Stores a secret associated with the given key.
    /// The secret does not have to be a string.
    fn store_secret(&self, key: ImmutableOrigin, secret: Vec<u8>) -> Result<(), String>;
    /// - `Ok(None)` if the secret does not exist
    /// - `Ok(Some(secret))` if the secret exists
    /// - `Err` if there was an error retrieving the secret
    fn retrieve_secret(&self, key: ImmutableOrigin) -> Result<Option<Vec<u8>>, String>;
    /// Deletes the secret associated with the given key.
    /// If the secret does not exist, this should still be treated as a success.
    /// This should only return `Err` if there was an error during deletion.
    fn delete_secret(&self, key: ImmutableOrigin) -> Result<(), String>;
}

pub struct DefaultCredentialManagementDelegate {
    secrets: Mutex<HashMap<ImmutableOrigin, Vec<u8>>>,
}

impl DefaultCredentialManagementDelegate {
    pub fn new() -> Self {
        DefaultCredentialManagementDelegate {
            secrets: Mutex::new(HashMap::new()),
        }
    }
}

impl CredentialManagementDelegate for DefaultCredentialManagementDelegate {
    fn store_secret(&self, key: ImmutableOrigin, secret: Vec<u8>) -> Result<(), String> {
        let mut secrets = self.secrets.lock().map_err(|e| e.to_string())?;
        secrets.insert(key, secret);
        Ok(())
    }

    fn retrieve_secret(&self, key: ImmutableOrigin) -> Result<Option<Vec<u8>>, String> {
        let secrets = self.secrets.lock().map_err(|e| e.to_string())?;
        if let Some(secret) = secrets.get(&key) {
            Ok(Some(secret.clone()))
        } else {
            Ok(None)
        }
    }

    fn delete_secret(&self, key: ImmutableOrigin) -> Result<(), String> {
        let mut secrets = self.secrets.lock().map_err(|e| e.to_string())?;
        secrets.remove(&key);
        Ok(())
    }
}
