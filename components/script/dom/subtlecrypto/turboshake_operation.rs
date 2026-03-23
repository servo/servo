/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use digest::{ExtendableOutput, Update};
use sha3::{TurboShake128, TurboShake128Core, TurboShake256, TurboShake256Core};

use crate::dom::bindings::error::Error;
use crate::dom::subtlecrypto::{CryptoAlgorithm, SubtleTurboShakeParams};

/// <https://wicg.github.io/webcrypto-modern-algos/#turboshake-operations-digest>
pub(crate) fn digest(
    normalized_algorithm: &SubtleTurboShakeParams,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. Let outputLength be the outputLength member of normalizedAlgorithm.
    let output_length = normalized_algorithm.output_length;

    // Step 2. If outputLength is zero or is not a multiple of 8, then throw an OperationError.
    if output_length == 0 || output_length % 8 != 0 {
        return Err(Error::Operation(Some(
            "The outputLength is zero or is not a multiple of 8".to_string(),
        )));
    }

    // Step 3. Let domainSeparation be the domainSeparation member of normalizedAlgorithm if
    // present, or 0x1F otherwise.
    let domain_separation = normalized_algorithm.domain_separation.unwrap_or(0x1f);

    // Step 4. If domainSeparation is less than 0x01 or greater than 0x7F, then throw an
    // OperationError.
    if !(0x01..=0x7f).contains(&domain_separation) {
        return Err(Error::Operation(Some(
            "The domainSeparation is less than 0x01 or greater than 0x7F".to_string(),
        )));
    }

    // Step 5.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for
    // "TurboSHAKE128":
    //     Let result be the result of performing the TurboSHAKE128 function defined in Section 2
    //     of [RFC9861] using message as the M input parameter, domainSeparation as the D input
    //     parameter, and outputLength divided by 8 as the L input parameter.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for
    // "TurboSHAKE256":
    //     Let result be the result of performing the TurboSHAKE256 function defined in Section 2
    //     of [RFC9861] using message as the M input parameter, domainSeparation as the D input
    //     parameter, and outputLength divided by 8 as the L input parameter.
    // Step 6. If performing the operation results in an error, then throw an OperationError.
    let result = match normalized_algorithm.name {
        CryptoAlgorithm::TurboShake128 => {
            let core = TurboShake128Core::new(domain_separation);
            let mut hasher = TurboShake128::from_core(core);
            hasher.update(message);
            hasher.finalize_boxed(output_length as usize / 8).to_vec()
        },
        CryptoAlgorithm::TurboShake256 => {
            let core = TurboShake256Core::new(domain_separation);
            let mut hasher = TurboShake256::from_core(core);
            hasher.update(message);
            hasher.finalize_boxed(output_length as usize / 8).to_vec()
        },
        algorithm_name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not a TurboSHAKE algorithm",
                algorithm_name.as_str()
            ))));
        },
    };

    // Step 7. Return result.
    Ok(result)
}
