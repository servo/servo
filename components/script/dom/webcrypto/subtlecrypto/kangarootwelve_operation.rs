/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use k12::{CustomRefKt128, CustomRefKt256, ExtendableOutput, Update};

use crate::dom::bindings::error::Error;
use crate::dom::subtlecrypto::{CryptoAlgorithm, SubtleKangarooTwelveParams};

/// <https://wicg.github.io/webcrypto-modern-algos/#kangarootwelve-operations-digest>
pub(crate) fn digest(
    normalized_algorithm: &SubtleKangarooTwelveParams,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. Let outputLength be the outputLength member of normalizedAlgorithm.
    let output_length = normalized_algorithm.output_length;

    // Step 2. If outputLength is zero or is not a multiple of 8, then throw an OperationError.
    if output_length == 0 || !output_length.is_multiple_of(8) {
        return Err(Error::Operation(Some(
            "The outputLength is zero or is not a multiple of 8".into(),
        )));
    }

    // Step 3. Let customization be the customization member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let customization = normalized_algorithm
        .customization
        .as_deref()
        .unwrap_or_default();

    // Step 4.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "KT128":
    //     Let result be the result of performing the KT128 function defined in Section 3 of
    //     [RFC9861] using message as the M input parameter, customization as the C input parameter,
    //     and outputLength divided by 8 as the L input parameter.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "KT256":
    //     Let result be the result of performing the KT256 function defined in Section 3 of
    //     [RFC9861] using message as the M input parameter, customization as the C input parameter,
    //     and outputLength divided by 8 as the L input parameter.
    // Step 5. If performing the operation results in an error, then throw an OperationError.
    let mut result = vec![0u8; output_length as usize / 8];
    match normalized_algorithm.name {
        CryptoAlgorithm::Kt128 => {
            let mut hasher = CustomRefKt128::new_customized(customization);
            hasher.update(message);
            hasher.finalize_xof_into(&mut result);
        },
        CryptoAlgorithm::Kt256 => {
            let mut hasher = CustomRefKt256::new_customized(customization);
            hasher.update(message);
            hasher.finalize_xof_into(&mut result);
        },
        algorithm_name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not supported",
                algorithm_name.as_str()
            ))));
        },
    }

    // Step 6. Return result.
    Ok(result)
}
