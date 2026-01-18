/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use digest::{ExtendableOutput, Update};
use sha3::{CShake128, CShake128Core, CShake256, CShake256Core};

use crate::dom::bindings::error::Error;
use crate::dom::subtlecrypto::{ALG_CSHAKE_128, ALG_CSHAKE_256, SubtleCShakeParams};

/// <https://wicg.github.io/webcrypto-modern-algos/#cshake-operations-digest>
pub(crate) fn digest(
    normalized_algorithm: &SubtleCShakeParams,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. Let length be the length member of normalizedAlgorithm.
    let length = normalized_algorithm.length as usize;

    // Step 2. Let functionName be the functionName member of normalizedAlgorithm if present or the
    // empty octet string otherwise.
    let function_name = normalized_algorithm.function_name.as_deref().unwrap_or(&[]);

    // Step 3. Let customization be the customization member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let customization = normalized_algorithm.customization.as_deref().unwrap_or(&[]);

    // Step 4.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "cSHAKE128":
    //     Let result be the result of performing the cSHAKE128 function defined in Section 3 of
    //     [NIST-SP800-185] using message as the X input parameter, length as the L input
    //     parameter, functionName as the N input parameter, and customization as the S input
    //     parameter.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "cSHAKE256":
    //     Let result be the result of performing the cSHAKE256 function defined in Section 3 of
    //     [NIST-SP800-185] using message as the X input parameter, length as the L input
    //     parameter, functionName as the N input parameter, and customization as the S input
    //     parameter.
    // Step 5. If performing the operation results in an error, then throw an OperationError.
    let result = match normalized_algorithm.name.as_str() {
        ALG_CSHAKE_128 => {
            let core = CShake128Core::new_with_function_name(function_name, customization);
            let mut hasher = CShake128::from_core(core);
            hasher.update(message);
            hasher.finalize_boxed(length / 8).to_vec()
        },
        ALG_CSHAKE_256 => {
            let core = CShake256Core::new_with_function_name(function_name, customization);
            let mut hasher = CShake256::from_core(core);
            hasher.update(message);
            hasher.finalize_boxed(length / 8).to_vec()
        },
        algorithm_name => {
            return Err(Error::NotSupported(Some(format!(
                "{algorithm_name} is not supported"
            ))));
        },
    };

    // Step 6. Return result.
    Ok(result)
}
