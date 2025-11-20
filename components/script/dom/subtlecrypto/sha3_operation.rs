/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use sha3::{Digest, Sha3_256, Sha3_384, Sha3_512};

use crate::dom::bindings::error::Error;
use crate::dom::subtlecrypto::{ALG_SHA3_256, ALG_SHA3_384, ALG_SHA3_512, SubtleAlgorithm};

/// <https://wicg.github.io/webcrypto-modern-algos/#sha3-operations-digest>
pub(crate) fn digest(
    normalized_algorithm: &SubtleAlgorithm,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "SHA3-256":
    //     Let result be the result of performing the SHA3-256 hash function defined in Section 6.1
    //     of [FIPS-202] using message as the input message, M.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "SHA3-384":
    //     Let result be the result of performing the SHA3-384 hash function defined in Section 6.1
    //     of [FIPS-202] using message as the input message, M.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "SHA3-512":
    //     Let result be the result of performing the SHA3-512 hash function defined in Section 6.1
    //     of [FIPS-202] using message as the input message, M.
    // Step 2. If performing the operation results in an error, then throw an OperationError.
    let result = match normalized_algorithm.name.as_str() {
        ALG_SHA3_256 => Sha3_256::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA3_384 => Sha3_384::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA3_512 => Sha3_512::new_with_prefix(message).finalize().to_vec(),
        _ => return Err(Error::NotSupported(None)),
    };

    // Step 3. Return result.
    Ok(result)
}
