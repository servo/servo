/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::collections::HashMap;

use servo_url::ImmutableOrigin;

pub trait CredentialManagementDelegate {
    /// Stores a secret associated with the given key.
    /// The secret does not have to be a string.
    fn store(&self, key: ImmutableOrigin, secret: Vec<u8>) -> Result<(), String>;
    /// Retrieve a secret if it exists.
    /// - `Ok(None)` if the secret does not exist
    /// - `Ok(Some(secret))` if the secret exists
    /// - `Err` if there was an error retrieving the secret
    fn retrieve(&self, key: ImmutableOrigin) -> Result<Option<Vec<u8>>, String>;
}

#[derive(Default)]
pub struct DefaultCredentialManagementDelegate {
    secrets: RefCell<HashMap<ImmutableOrigin, Vec<u8>>>,
}

impl CredentialManagementDelegate for DefaultCredentialManagementDelegate {
    fn store(&self, key: ImmutableOrigin, secret: Vec<u8>) -> Result<(), String> {
        self.secrets.borrow_mut().insert(key, secret);
        Ok(())
    }

    fn retrieve(&self, key: ImmutableOrigin) -> Result<Option<Vec<u8>>, String> {
        Ok(self.secrets.borrow().get(&key).cloned())
    }
}
