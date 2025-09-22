/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aws_lc_rs::digest;

use crate::dom::bindings::error::Error;
use crate::dom::subtlecrypto::{ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, SubtleAlgorithm};

/// <https://w3c.github.io/webcrypto/#sha-operations-digest>
pub(crate) fn digest(
    nomrmalized_algorithm: &SubtleAlgorithm,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1.
    // If the name member of normalizedAlgorithm is a cases-sensitive string match for "SHA-1":
    //     Let result be the result of performing the SHA-1 hash function defined in Section 6.1 of
    //     [FIPS-180-4] using message as the input message, M.
    // If the name member of normalizedAlgorithm is a cases-sensitive string match for "SHA-256":
    //     Let result be the result of performing the SHA-256 hash function defined in Section 6.2
    //     of [FIPS-180-4] using message as the input message, M.
    // If the name member of normalizedAlgorithm is a cases-sensitive string match for "SHA-384":
    //     Let result be the result of performing the SHA-384 hash function defined in Section 6.5
    //     of [FIPS-180-4] using message as the input message, M.
    // If the name member of normalizedAlgorithm is a cases-sensitive string match for "SHA-512":
    //     Let result be the result of performing the SHA-512 hash function defined in Section 6.4
    //     of [FIPS-180-4] using message as the input message, M.
    // Step 2. If performing the operation results in an error, then throw an OperationError.
    let result = match nomrmalized_algorithm.name.as_str() {
        ALG_SHA1 => digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, message)
            .as_ref()
            .to_vec(),
        ALG_SHA256 => digest::digest(&digest::SHA256, message).as_ref().to_vec(),
        ALG_SHA384 => digest::digest(&digest::SHA384, message).as_ref().to_vec(),
        ALG_SHA512 => digest::digest(&digest::SHA512, message).as_ref().to_vec(),
        _ => {
            return Err(Error::NotSupported);
        },
    };

    // Step 3. Return result.
    Ok(result)
}
