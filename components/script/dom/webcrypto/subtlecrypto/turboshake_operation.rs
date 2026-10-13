/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use keccak::{Keccak, State1600};
use sponge_cursor::SpongeCursor;

use crate::dom::bindings::error::Error;
use crate::dom::subtlecrypto::{CryptoAlgorithm, TurboShakeParams};

/// <https://wicg.github.io/webcrypto-modern-algos/#turboshake-operations-digest>
pub(crate) fn digest(
    normalized_algorithm: &TurboShakeParams,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. Let outputLength be the outputLength member of normalizedAlgorithm.
    let output_length = normalized_algorithm.output_length;

    // Step 2. If outputLength is zero or is not a multiple of 8, then throw an OperationError.
    if output_length == 0 || !output_length.is_multiple_of(8) {
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
    let mut result = vec![0u8; output_length as usize / 8];
    match normalized_algorithm.name {
        CryptoAlgorithm::TurboShake128 => {
            let hasher = TurboShake::<168>::new(domain_separation)?;
            hasher.hash(message, &mut result);
        },
        CryptoAlgorithm::TurboShake256 => {
            let hasher = TurboShake::<136>::new(domain_separation)?;
            hasher.hash(message, &mut result);
        },
        algorithm_name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not a TurboSHAKE algorithm",
                algorithm_name.as_str()
            ))));
        },
    }

    // Step 7. Return result.
    Ok(result)
}

/// Keccak rounds for TurboSHAKE
const ROUNDS: usize = 12;

/// TurboSHAKE hasher. RATE must be either 168 for TurboSHAKE128 or 136 for TurboSHAKE256.
struct TurboShake<const RATE: usize> {
    state: State1600,
    cursor: SpongeCursor<RATE>,
    keccak: Keccak,
    domain_separation: u8,
}

impl<const RATE: usize> TurboShake<RATE> {
    fn new(domain_separation: u8) -> Result<TurboShake<RATE>, Error> {
        if RATE != 168 && RATE != 136 {
            return Err(Error::NotSupported(Some(
                "Invalid sponge cursor rate for TurboSHAKE".into(),
            )));
        }
        if domain_separation == 0x00 || domain_separation > 0x7f {
            return Err(Error::NotSupported(Some(
                "Invalid TurboSHAKE domain separation".into(),
            )));
        }
        Ok(TurboShake {
            state: Default::default(),
            cursor: Default::default(),
            keccak: Default::default(),
            domain_separation,
        })
    }

    fn hash(mut self, message: &[u8], result: &mut [u8]) {
        // Digest message
        //
        // Reference implementation from RustCrypto:
        // <https://docs.rs/turboshake/0.7.1/src/turboshake/lib.rs.html#60-64>
        self.keccak.with_p1600::<ROUNDS>(|p1600| {
            self.cursor.absorb_u64_le(&mut self.state, p1600, message);
        });

        // Finalize
        //
        // Reference implementation from RustCrypto:
        // <https://docs.rs/turboshake/0.7.1/src/turboshake/lib.rs.html#67-91>
        let position = self.cursor.pos();
        self.state[position / 8] ^= (self.domain_separation as u64) << (8 * (position % 8));
        self.state[RATE / 8 - 1] ^= 1 << 63;

        // Read result
        //
        // Reference implementation from RustCrypto:
        // <https://docs.rs/turboshake/0.7.1/src/turboshake/lib.rs.html#161-165>
        self.cursor = Default::default();
        self.keccak.with_p1600::<ROUNDS>(|p1600| {
            self.cursor
                .squeeze_read_u64_le(&mut self.state, p1600, result);
        });
    }
}
