/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use keyring::Entry;

pub trait CredentialManagementDelegate {
    /// Stores a secret associated with the given key.
    /// The secret does not have to be a string.
    fn store_secret(&self, key: &str, secret: Vec<u8>) -> Result<(), String>;
    /// - `Ok(None)` if the secret does not exist
    /// - `Ok(Some(secret))` if the secret exists
    /// - `Err` if there was an error retrieving the secret
    fn retrieve_secret(&self, key: &str) -> Result<Option<Vec<u8>>, String>;
    /// Deletes the secret associated with the given key.
    /// If the secret does not exist, this should still be treated as a success.
    /// This should only return `Err` if there was an error during deletion.
    fn delete_secret(&self, key: &str) -> Result<(), String>;
}

pub struct DefaultCredentialManagementDelegate {
    service_name: String,
}

impl DefaultCredentialManagementDelegate {
    pub fn new(service_name: String) -> Self {
        Self { service_name  }
    }

    /// Helper to get a keyring entry for a given key
    fn get_entry(&self, user: &str) -> Result<Entry, String> {
        Entry::new(&self.service_name, user).map_err(|e| format!("Failed to create keyring entry: {}", e))
    }
}

impl CredentialManagementDelegate for DefaultCredentialManagementDelegate {
    fn store_secret(&self, key: &str, secret: Vec<u8>) -> Result<(), String> {
        let entry = self.get_entry(key)?;
        entry.set_secret(&secret).map_err(|e| format!("Failed to store secret: {}", e))
    }

    fn retrieve_secret(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let entry = self.get_entry(key)?;
        match entry.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("Failed to retrieve secret: {}", e)),
        }
    }

    fn delete_secret(&self, key: &str) -> Result<(), String> {
        let entry = self.get_entry(key)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("Failed to delete secret: {}", e)),
        }
    }
}
