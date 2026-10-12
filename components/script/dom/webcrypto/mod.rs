/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod crypto {
    pub(crate) type Crypto = script_webcrypto::crypto::Crypto<crate::DomTypeHolder>;
}
#[cfg(feature = "subtlecrypto")]
pub(crate) mod cryptokey;
#[cfg(feature = "subtlecrypto")]
pub(crate) mod subtlecrypto;
