/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cshake::CShake;
use cshake::digest::{ExtendableOutput, Update};
use ctutils::CtEq;
use js::context::JSContext;
use rand::TryRng;
use zeroize::Zeroizing;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    CryptoAlgorithm, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    SubtleKmacImportParams, SubtleKmacKeyAlgorithm, SubtleKmacKeyGenParams, SubtleKmacParams,
};

/// <https://wicg.github.io/webcrypto-modern-algos/#kmac-operations-sign>
pub(crate) fn sign(
    normalized_algorithm: &SubtleKmacParams,
    key: &CryptoKey,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. Let customization be the customization member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let customization = normalized_algorithm
        .customization
        .as_deref()
        .unwrap_or_default();

    // Step 2.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "KMAC128":
    //     Let mac be the result of performing the KMAC128 function defined in Section 4 of
    //     [NIST-SP800-185] using the key represented by [[handle]] internal slot of key as the K
    //     input parameter, message as the X input parameter, the outputLength member of
    //     normalizedAlgorithm as the L input parameter, and customization as the S input parameter.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "KMAC256":
    //     Let mac be the result of performing the KMAC256 function defined in Section 4 of
    //     [NIST-SP800-185] using the key represented by [[handle]] internal slot of key as the K
    //     input parameter, message as the X input parameter, the outputLength member of
    //     normalizedAlgorithm as the L input parameter, and customization as the S input parameter.
    let Handle::KmacKey(kmac_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not a KMAC key".into(),
        )));
    };
    let KeyAlgorithmAndDerivatives::KmacKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(Some(
            "[[algorithm]] internal slot of key is not a KmacKeyAlgorithm".into(),
        )));
    };
    let mac = match normalized_algorithm.name {
        CryptoAlgorithm::Kmac128 => kmac128(
            kmac_key,
            algorithm.length,
            message,
            normalized_algorithm.output_length,
            customization,
        ),
        CryptoAlgorithm::Kmac256 => kmac256(
            kmac_key,
            algorithm.length,
            message,
            normalized_algorithm.output_length,
            customization,
        ),
        algorithm_name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not supported",
                algorithm_name.as_str()
            ))));
        },
    };

    // Step 3. Return a byte sequence containing mac.
    Ok(mac)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#kmac-operations-verify>
pub(crate) fn verify(
    normalized_algorithm: &SubtleKmacParams,
    key: &CryptoKey,
    message: &[u8],
    signature: &[u8],
) -> Result<bool, Error> {
    // Step 1. Let customization be the customization member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let customization = normalized_algorithm
        .customization
        .as_deref()
        .unwrap_or_default();

    // Step 2.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "KMAC128":
    //     Let mac be the result of performing the KMAC128 function defined in Section 4 of
    //     [NIST-SP800-185] using the key represented by [[handle]] internal slot of key as the K
    //     input parameter, message as the X input parameter, the outputLength member of
    //     normalizedAlgorithm as the L input parameter, and customization as the S input parameter.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "KMAC256":
    //     Let mac be the result of performing the KMAC256 function defined in Section 4 of
    //     [NIST-SP800-185] using the key represented by [[handle]] internal slot of key as the K
    //     input parameter, message as the X input parameter, the outputLength member of
    //     normalizedAlgorithm as the L input parameter, and customization as the S input parameter.
    // Step 3. Let computedMac be a byte sequence containing mac.
    let Handle::KmacKey(kmac_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not a KMAC key".into(),
        )));
    };
    let KeyAlgorithmAndDerivatives::KmacKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(Some(
            "[[algorithm]] internal slot of key is not a KmacKeyAlgorithm".into(),
        )));
    };
    let computed_mac = match normalized_algorithm.name {
        CryptoAlgorithm::Kmac128 => kmac128(
            kmac_key,
            algorithm.length,
            message,
            normalized_algorithm.output_length,
            customization,
        ),
        CryptoAlgorithm::Kmac256 => kmac256(
            kmac_key,
            algorithm.length,
            message,
            normalized_algorithm.output_length,
            customization,
        ),
        algorithm_name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not supported",
                algorithm_name.as_str()
            ))));
        },
    };

    // Step 4. Return true if computedMac is equal to signature and false otherwise. This comparison
    // must be performed in constant-time.
    Ok(computed_mac.ct_eq(signature).into())
}

/// <https://wicg.github.io/webcrypto-modern-algos/#kmac-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleKmacKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If usages contains an entry which is not "sign" or "verify", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"sign\" or \"verify\"".into(),
        )));
    }

    // Step 2.
    // If the length member of normalizedAlgorithm is present:
    //     Let length be equal to the length member of normalizedAlgorithm.
    // Otherwise, if the name member of normalizedAlgorithm is a case-sensitive string match for
    // "KMAC128":
    //     Let length be 128.
    // Otherwise, if the name member of normalizedAlgorithm is a case-sensitive string match for
    // "KMAC256":
    //     Let length be 256.
    let length = if let Some(normalized_algorithm_length) = normalized_algorithm.length {
        normalized_algorithm_length
    } else if normalized_algorithm.name == CryptoAlgorithm::Kmac128 {
        128
    } else if normalized_algorithm.name == CryptoAlgorithm::Kmac256 {
        256
    } else {
        return Err(Error::Data(Some(
            "Unable to determine the length of KMAC key".into(),
        )));
    };

    // Step 3. Generate a key of length length bits.
    // Step 4. If the key generation step fails, then throw an OperationError.
    //
    // NOTE: We store the KMAC key bits as the byte sequence containing bits.
    // <https://w3c.github.io/webcrypto/#dfn-byte-sequence-containing>
    let mut rng = rand::rng();
    let mut bits = vec![0u8; length.div_ceil(8) as usize];
    rng.try_fill_bytes(&mut bits)
        .map_err(|_| Error::Operation(Some("Failed to generate KMAC key".into())))?;
    if !length.is_multiple_of(8) {
        // Clean excess bits in the last byte of result.
        let mask = u8::MAX << (8 - length % 8);
        if let Some(last_byte) = bits.last_mut() {
            *last_byte &= mask;
        }
    }

    // Step 5. Let key be a new CryptoKey object representing the generated key.
    // Step 6. Set the [[type]] internal slot of key to "secret".
    // Step 7. Let algorithm be a new KmacKeyAlgorithm.
    // Step 8. Set the name attribute of algorithm to the name member of normalizedAlgorithm.
    // Step 9. Set the length attribute of algorithm to length.
    // Step 10. Set the [[algorithm]] internal slot of key to algorithm.
    // Step 11. Set the [[extractable]] internal slot of key to be extractable.
    // Step 12. Set the [[usages]] internal slot of key to be usages.
    let algorithm = SubtleKmacKeyAlgorithm {
        name: normalized_algorithm.name,
        length,
    };
    let key = CryptoKey::new(
        cx,
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KmacKeyAlgorithm(algorithm),
        usages,
        Handle::KmacKey(bits.into()),
    );

    // Step 13. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#kmac-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleKmacImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.
    // NOTE: It is given as a method parameter.

    // Step 2. If usages contains an entry which is not "sign" or "verify", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"sign\" or \"verify\"".into(),
        )));
    }

    // Step 3.
    let mut data: Zeroizing<Vec<u8>>;
    match format {
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 3.1. Let data be keyData.
            data = key_data.to_vec().into();
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(cx, key_data)?;

            // Step 3.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"oct\"".into(),
                )));
            }

            // Step 3.3. If jwk does not meet the requirements of Section 6.4 of JSON Web Algorithms
            // [JWA], then throw a DataError.
            // Step 3.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = jwk.decode_required_string_field(JwkStringField::K)?;

            // Step 3.4.
            // If the name member of normalizedAlgorithm is a case-sensitive string match for
            // "KMAC128":
            //     If the alg field of jwk is present and is not "K128", then throw a DataError.
            // If the name member of normalizedAlgorithm is a case-sensitive string match for
            // "KMAC256":
            //     If the alg field of jwk is present and is not "K256", then throw a DataError.
            if normalized_algorithm.name == CryptoAlgorithm::Kmac128 &&
                jwk.alg.as_ref().is_some_and(|alg| alg != "K128")
            {
                return Err(Error::Data(Some(
                    "The alg field of jwk is present and is not \"K128\"".into(),
                )));
            }
            if normalized_algorithm.name == CryptoAlgorithm::Kmac256 &&
                jwk.alg.as_ref().is_some_and(|alg| alg != "K256")
            {
                return Err(Error::Data(Some(
                    "The alg field of jwk is present and is not \"K256\"".into(),
                )));
            }

            // Step 3.6. If usages is non-empty and the use field of jwk is present and is not
            // "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not \
                    equal to \"sig\""
                        .into(),
                )));
            }

            // Step 3.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 3.8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some(
                    "The ext field of jwk is present and has the value false and extractable \
                    is true"
                        .into(),
                )));
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported import key format for KMAC key".into(),
            )));
        },
    }

    // Step 4. Let length be the length in bits of data.
    let mut length = data.len() as u32 * 8;

    // Step 5.
    // If the length member of normalizedAlgorithm is present:
    //     If the length member of normalizedAlgorithm is greater than length:
    //         throw a DataError.
    //     If the length member of normalizedAlgorithm, is less than or equal to length minus eight:
    //         throw a DataError.
    //     Otherwise:
    //         Set length equal to the length member of normalizedAlgorithm.
    if let Some(normalized_algorithm_length) = normalized_algorithm.length {
        if normalized_algorithm_length > length {
            return Err(Error::Data(Some("The key bit string is too short".into())));
        } else if normalized_algorithm_length + 8 <= length {
            return Err(Error::Data(Some("The key bit string is too long".into())));
        } else {
            length = normalized_algorithm_length;
        }
    }

    // Step 6. Let key be a new CryptoKey object representing an KMAC key with the first length bits
    // of data.
    // Step 7. Set the [[type]] internal slot of key to "secret".
    // Step 8. Let algorithm be a new KmacKeyAlgorithm.
    // Step 9. Set the name attribute of algorithm to the name member of normalizedAlgorithm.
    // Step 10. Set the length attribute of algorithm to length.
    // Step 11. Set the [[algorithm]] internal slot of key to algorithm.
    //
    // NOTE: We store the first length bits of data as the byte sequence containing bits.
    // <https://w3c.github.io/webcrypto/#dfn-byte-sequence-containing>
    if !length.is_multiple_of(8) {
        // Clean excess bits in the last byte of result.
        let mask = u8::MAX << (8 - length % 8);
        if let Some(last_byte) = data.last_mut() {
            *last_byte &= mask;
        }
    }
    let algorithm = SubtleKmacKeyAlgorithm {
        name: normalized_algorithm.name,
        length,
    };
    let key = CryptoKey::new(
        cx,
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KmacKeyAlgorithm(algorithm),
        usages,
        Handle::KmacKey(data),
    );

    // Step 12. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#kmac-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.

    // Step 2. Let bits be the raw bits of the key represented by [[handle]] internal slot of key.
    // Step 3. Let data be an byte sequence containing bits.
    //
    // NOTE: We already store KMAC key bits as the byte sequence containing bits in the [[handle]]
    // internal slot of key.
    let Handle::KmacKey(data) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not a KMAC key".into(),
        )));
    };

    // Step 4.
    let result = match format {
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 4.1. Let result be data.
            ExportedKey::new_bytes(data.to_vec())
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 4.1. Let jwk be a new JsonWebKey dictionary.
            let mut jwk = JsonWebKey::default();

            // Step 4.2. Set the kty attribute of jwk to the string "oct".
            jwk.kty = Some(DOMString::from("oct"));

            // Step 4.3. Set the k attribute of jwk to be a string containing data, encoded
            // according to Section 6.4 of JSON Web Algorithms [JWA].
            jwk.encode_string_field(JwkStringField::K, data);

            // Step 4.4. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KmacKeyAlgorithm(key_algorithm) = key.algorithm()
            else {
                return Err(Error::Operation(Some("The key is not a KMAC key".into())));
            };

            // Step 4.5.
            // If the name member of keyAlgorithm is "KMAC128":
            //     Set the alg attribute of jwk to the string "K128".
            // If the name member of keyAlgorithm is "KMAC256":
            //     Set the alg attribute of jwk to the string "K256".
            if key_algorithm.name == CryptoAlgorithm::Kmac128 {
                jwk.alg = Some(DOMString::from("K128"));
            }
            if key_algorithm.name == CryptoAlgorithm::Kmac256 {
                jwk.alg = Some(DOMString::from("K256"));
            }

            // Step 4.6. Set the key_ops attribute of jwk to equal the usages attribute of key.
            jwk.set_key_ops(&key.usages());

            // Step 4.7. Set the ext attribute of jwk to equal the [[extractable]] internal slot of
            // key.
            jwk.ext = Some(key.Extractable());

            // Step 4.8. Let result be jwk.
            ExportedKey::new_jwk(jwk)
        },
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported import key format for KMAC".into(),
            )));
        },
    };

    // Step 5. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#kmac-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_algorithm: &SubtleKmacImportParams,
) -> Result<Option<u32>, Error> {
    // Step 1.
    // If the length member of normalizedAlgorithm is present:
    //     Let length be equal to the length member of normalizedAlgorithm.
    // Otherwise, if the name member of normalizedAlgorithm is a case-sensitive string match for
    // "KMAC128":
    //     Let length be 128.
    // Otherwise, if the name member of normalizedAlgorithm is a case-sensitive string match for
    // "KMAC256":
    //     Let length be 256.
    let length = if let Some(normalized_algorithm_length) = normalized_algorithm.length {
        normalized_algorithm_length
    } else if normalized_algorithm.name == CryptoAlgorithm::Kmac128 {
        128
    } else if normalized_algorithm.name == CryptoAlgorithm::Kmac256 {
        256
    } else {
        return Err(Error::Data(Some(
            "Unable to determine the length of KMAC key".into(),
        )));
    };

    // Step 2. Return length.
    Ok(Some(length))
}

/// The KMAC128 function defined in Section 4 of [NIST-SP800-185], using `key` as the K input
/// parameter, `message` as the X input parameter, `output_length` as the L input parameter, and
/// `customization` as the S input parameter, where `key_length` is the key length in bits.
/// <https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-185.pdf>
///
/// Since the key is represented as a byte sequence, and the key length in bits might not be a
/// muliple of 8, callers need to provide the actual key length in bits via the function argument
/// `key_length`, and the excess bits at the end of the byte sequence must be zeros.
fn kmac128(
    key: &[u8],
    key_length: u32,
    message: &[u8],
    output_length: u32,
    customization: &[u8],
) -> Vec<u8> {
    kmac::<168>(key, key_length, message, output_length, customization)
}

/// The KMAC256 function defined in Section 4 of [NIST-SP800-185], using `key` as the K input
/// parameter, `message` as the X input parameter, `output_length` as the L input parameter, and
/// `customization` as the S input parameter, where `key_length` is the key length in bits.
/// <https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-185.pdf>
///
/// Since the key is represented as a byte sequence, and the key length in bits might not be a
/// muliple of 8, callers need to provide the actual key length in bits via the function argument
/// `key_length`, and the excess bits at the end of the byte sequence must be zeros.
fn kmac256(
    key: &[u8],
    key_length: u32,
    message: &[u8],
    output_length: u32,
    customization: &[u8],
) -> Vec<u8> {
    kmac::<136>(key, key_length, message, output_length, customization)
}

/// A shared implementation for both the KMAC128 function and KMAC256 function defined in Section 4
/// of [NIST-SP800-185], using `key` as the K input parameter, `message` as the X input parameter,
/// `output_length` as the L input parameter, and `customization` as the S input parameter, where
/// `key_length` is the key length in bits.
/// <https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-185.pdf>
///
/// To be specific, this function implements the following algorithm, where RATE must be either 168
/// or 136 for KMAC128 and KMAC256 respectively.
///
/// 1. newX = bytepad(encode_string(K), RATE) || X || right_encode(L).
/// 2. return cSHAKE128(newX, L, “KMAC”, S).
///
/// Since the key is represented as a byte sequence, and the key length in bits might not be a
/// muliple of 8, callers need to provide the actual key length in bits via the function argument
/// `key_length`, and the excess bits at the end of the byte sequence must be zeros.
fn kmac<const RATE: usize>(
    key: &[u8],
    key_length: u32,
    message: &[u8],
    output_length: u32,
    customization: &[u8],
) -> Vec<u8> {
    let mut buffer = [0u8; 5];
    let zeros = [0u8; RATE];

    // Initialize cSHAKE.
    let mut cshake = CShake::<RATE>::new_with_function_name(b"KMAC", customization);

    // Hash bytepad(encode_string(K), RATE).
    let update_and_return_written = |cshake: &mut CShake<RATE>, input: &[u8]| -> usize {
        cshake.update(input);
        input.len()
    };
    let mut written = update_and_return_written(&mut cshake, left_encode(RATE as u32, &mut buffer));
    written += update_and_return_written(&mut cshake, left_encode(key_length, &mut buffer));
    written += update_and_return_written(&mut cshake, key);
    cshake.update(&zeros[0..RATE - written % RATE]);

    // Hash X.
    cshake.update(message);

    // Hash right_encode(L).
    cshake.update(right_encode(output_length, &mut buffer));

    // Finalize cSHAKE.
    let mut result = vec![0u8; output_length.div_ceil(8) as usize];
    cshake.finalize_xof_into(&mut result);
    if !output_length.is_multiple_of(8) {
        // Clean excess bits in the last byte of result.
        let mask = u8::MAX << (8 - output_length % 8);
        if let Some(last_byte) = result.last_mut() {
            *last_byte &= mask;
        }
    }

    result
}

/// The right_encode function defined in Section 2.3.1 of [NIST-SP800-185], using `value` as the x
/// input parameter. <https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-185.pdf>
///
/// Callers need to provide `buffer` for internal operations. The buffer can be reused afterwards in
/// separate calls of [`right_encode`] and [`left_encode`].
fn right_encode(value: u32, buffer: &mut [u8; 5]) -> &[u8] {
    buffer[0..4].copy_from_slice(&value.to_be_bytes());
    let leading_zero = buffer[0..3].iter().take_while(|byte| **byte == 0).count();
    buffer[4] = u8::try_from(4 - leading_zero).expect("The number must be 1..=4");
    &buffer[leading_zero..5]
}

/// The left_encode function defined in Section 2.3.1 of [NIST-SP800-185], using `value` as the x
/// input parameter. <https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-185.pdf>
///
/// Callers need to provide `buffer` for internal operations. The buffer can be reused afterwards in
/// separate calls of [`right_encode`] and [`left_encode`].
fn left_encode(value: u32, buffer: &mut [u8; 5]) -> &[u8] {
    buffer[1..5].copy_from_slice(&value.to_be_bytes());
    let leading_zero = buffer[1..4].iter().take_while(|byte| **byte == 0).count();
    buffer[leading_zero] = u8::try_from(4 - leading_zero).expect("The number must be 1..=4");
    &buffer[leading_zero..5]
}
