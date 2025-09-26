/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{KeyIvInit, StreamCipher};
use aes::{Aes128, Aes192, Aes256};

use crate::dom::bindings::error::Error;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::subtlecrypto::SubtleAesCtrParams;

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-encrypt>
pub(crate) fn encrypt_aes_ctr(
    params: &SubtleAesCtrParams,
    key: &CryptoKey,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the counter member of normalizedAlgorithm does not have a length of 16 bytes,
    // then throw an OperationError.
    // Step 2. If the length member of normalizedAlgorithm is zero or is greater than 128, then
    // throw an OperationError.
    if params.counter.len() != 16 || params.length == 0 || params.length > 128 {
        return Err(Error::Operation);
    }

    // Step 3. Let ciphertext be the result of performing the CTR Encryption operation described in
    // Section 6.5 of [NIST-SP800-38A] using AES as the block cipher, the counter member of
    // normalizedAlgorithm as the initial value of the counter block, the length member of
    // normalizedAlgorithm as the input parameter m to the standard counter block incrementing
    // function defined in Appendix B.1 of [NIST-SP800-38A] and plaintext as the input plaintext.
    let mut ciphertext = Vec::from(data);
    let counter = GenericArray::from_slice(&params.counter);
    match key.handle() {
        Handle::Aes128(data) => {
            let key_data = GenericArray::from_slice(data);
            ctr::Ctr64BE::<Aes128>::new(key_data, counter).apply_keystream(&mut ciphertext)
        },
        Handle::Aes192(data) => {
            let key_data = GenericArray::from_slice(data);
            ctr::Ctr64BE::<Aes192>::new(key_data, counter).apply_keystream(&mut ciphertext)
        },
        Handle::Aes256(data) => {
            let key_data = GenericArray::from_slice(data);
            ctr::Ctr64BE::<Aes256>::new(key_data, counter).apply_keystream(&mut ciphertext)
        },
        _ => return Err(Error::Data),
    };

    // Step 3. Return ciphertext.
    Ok(ciphertext)
}
