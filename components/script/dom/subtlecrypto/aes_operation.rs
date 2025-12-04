/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, StreamCipher};
use aes::{Aes128, Aes192, Aes256};
use aes_gcm::{AeadInPlace, AesGcm, KeyInit};
use aes_kw::{KekAes128, KekAes192, KekAes256};
use base64ct::{Base64UrlUnpadded, Encoding};
use cipher::consts::{U12, U16, U32};
use rand::TryRngCore;
use rand::rngs::OsRng;

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
    ALG_AES_CBC, ALG_AES_CTR, ALG_AES_GCM, ALG_AES_KW, ExportedKey, JsonWebKeyExt,
    KeyAlgorithmAndDerivatives, SubtleAesCbcParams, SubtleAesCtrParams, SubtleAesDerivedKeyParams,
    SubtleAesGcmParams, SubtleAesKeyAlgorithm, SubtleAesKeyGenParams,
};
use crate::script_runtime::CanGc;

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;
type Aes192CbcEnc = cbc::Encryptor<Aes192>;
type Aes192CbcDec = cbc::Decryptor<Aes192>;
type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;
type Aes128Ctr = ctr::Ctr64BE<Aes128>;
type Aes192Ctr = ctr::Ctr64BE<Aes192>;
type Aes256Ctr = ctr::Ctr64BE<Aes256>;

type Aes128Gcm96Iv = AesGcm<Aes128, U12>;
type Aes128Gcm128Iv = AesGcm<Aes128, U16>;
type Aes192Gcm96Iv = AesGcm<Aes192, U12>;
type Aes256Gcm96Iv = AesGcm<Aes256, U12>;
type Aes128Gcm256Iv = AesGcm<Aes128, U32>;
type Aes192Gcm256Iv = AesGcm<Aes192, U32>;
type Aes256Gcm256Iv = AesGcm<Aes256, U32>;

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-encrypt>
pub(crate) fn encrypt_aes_ctr(
    normalized_algorithm: &SubtleAesCtrParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the counter member of normalizedAlgorithm does not have a length of 16 bytes,
    // then throw an OperationError.
    if normalized_algorithm.counter.len() != 16 {
        return Err(Error::Operation(Some(
            "The initial counter block length is not 16 bytes".into(),
        )));
    }

    // Step 2. If the length member of normalizedAlgorithm is zero or is greater than 128, then
    // throw an OperationError.
    if normalized_algorithm.length == 0 {
        return Err(Error::Operation(Some("The counter length is zero".into())));
    }

    if normalized_algorithm.length > 128 {
        return Err(Error::Operation(Some(
            "The counter length is greater than 128".into(),
        )));
    }

    // Step 3. Let ciphertext be the result of performing the CTR Encryption operation described in
    // Section 6.5 of [NIST-SP800-38A] using AES as the block cipher, the counter member of
    // normalizedAlgorithm as the initial value of the counter block, the length member of
    // normalizedAlgorithm as the input parameter m to the standard counter block incrementing
    // function defined in Appendix B.1 of [NIST-SP800-38A] and plaintext as the input plaintext.
    let mut ciphertext = Vec::from(plaintext);
    let counter = GenericArray::from_slice(&normalized_algorithm.counter);

    match key.handle() {
        Handle::Aes128(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes128Ctr::new(key_data, counter).apply_keystream(&mut ciphertext)
        },
        Handle::Aes192(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes192Ctr::new(key_data, counter).apply_keystream(&mut ciphertext)
        },
        Handle::Aes256(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes256Ctr::new(key_data, counter).apply_keystream(&mut ciphertext)
        },
        _ => return Err(Error::Data(None)),
    };

    // Step 3. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-decrypt>
pub(crate) fn decrypt_aes_ctr(
    normalized_algorithm: &SubtleAesCtrParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // NOTE: Share implementation with `encrypt_aes_ctr`
    encrypt_aes_ctr(normalized_algorithm, key, ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-generate-key>
pub(crate) fn generate_key_aes_ctr(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    generate_key_aes(
        global,
        normalized_algorithm,
        extractable,
        usages,
        ALG_AES_CTR,
        &[
            KeyUsage::Encrypt,
            KeyUsage::Decrypt,
            KeyUsage::WrapKey,
            KeyUsage::UnwrapKey,
        ],
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-import-key>
pub(crate) fn import_key_aes_ctr(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    import_key_aes(
        global,
        format,
        key_data,
        extractable,
        usages,
        ALG_AES_CTR,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-export-key>
pub(crate) fn export_key_aes_ctr(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    export_key_aes(format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-get-key-length>
pub(crate) fn get_key_length_aes_ctr(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    get_key_length_aes(normalized_derived_key_algorithm)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-encrypt>
pub(crate) fn encrypt_aes_cbc(
    normalized_algorithm: &SubtleAesCbcParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 16 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 16 {
        return Err(Error::Operation(None));
    }

    // Step 2. Let paddedPlaintext be the result of adding padding octets to plaintext according to
    // the procedure defined in Section 10.3 of [RFC2315], step 2, with a value of k of 16.
    // Step 3. Let ciphertext be the result of performing the CBC Encryption operation described in
    // Section 6.2 of [NIST-SP800-38A] using AES as the block cipher, the iv member of
    // normalizedAlgorithm as the IV input parameter and paddedPlaintext as the input plaintext.
    let plaintext = Vec::from(plaintext);
    let iv = GenericArray::from_slice(&normalized_algorithm.iv);
    let ciphertext = match key.handle() {
        Handle::Aes128(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes128CbcEnc::new(key_data, iv).encrypt_padded_vec_mut::<Pkcs7>(&plaintext)
        },
        Handle::Aes192(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes192CbcEnc::new(key_data, iv).encrypt_padded_vec_mut::<Pkcs7>(&plaintext)
        },
        Handle::Aes256(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes256CbcEnc::new(key_data, iv).encrypt_padded_vec_mut::<Pkcs7>(&plaintext)
        },
        _ => return Err(Error::Data(None)),
    };

    // Step 4. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-decrypt>
pub(crate) fn decrypt_aes_cbc(
    normalized_algorithm: &SubtleAesCbcParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 16 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 16 {
        return Err(Error::Operation(None));
    }

    // Step 2. If the length of ciphertext is zero or is not a multiple of 16 bytes, then throw an
    // OperationError.
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(Error::Operation(None));
    }

    // Step 3. Let paddedPlaintext be the result of performing the CBC Decryption operation
    // described in Section 6.2 of [NIST-SP800-38A] using AES as the block cipher, the iv member of
    // normalizedAlgorithm as the IV input parameter and ciphertext as the input ciphertext.
    // Step 4. Let p be the value of the last octet of paddedPlaintext.
    // Step 5. If p is zero or greater than 16, or if any of the last p octets of paddedPlaintext
    // have a value which is not p, then throw an OperationError.
    // Step 6. Let plaintext be the result of removing p octets from the end of paddedPlaintext.
    let mut ciphertext = Vec::from(ciphertext);
    let iv = GenericArray::from_slice(&normalized_algorithm.iv);
    let plaintext = match key.handle() {
        Handle::Aes128(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes128CbcDec::new(key_data, iv)
                .decrypt_padded_mut::<Pkcs7>(ciphertext.as_mut_slice())
                .map_err(|_| Error::Operation(None))?
        },
        Handle::Aes192(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes192CbcDec::new(key_data, iv)
                .decrypt_padded_mut::<Pkcs7>(ciphertext.as_mut_slice())
                .map_err(|_| Error::Operation(None))?
        },
        Handle::Aes256(data) => {
            let key_data = GenericArray::from_slice(data);
            Aes256CbcDec::new(key_data, iv)
                .decrypt_padded_mut::<Pkcs7>(ciphertext.as_mut_slice())
                .map_err(|_| Error::Operation(None))?
        },
        _ => return Err(Error::Data(None)),
    };

    // Step 7. Return plaintext.
    Ok(plaintext.to_vec())
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-generate-key>
pub(crate) fn generate_key_aes_cbc(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    generate_key_aes(
        global,
        normalized_algorithm,
        extractable,
        usages,
        ALG_AES_CBC,
        &[
            KeyUsage::Encrypt,
            KeyUsage::Decrypt,
            KeyUsage::WrapKey,
            KeyUsage::UnwrapKey,
        ],
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-import-key>
pub(crate) fn import_key_aes_cbc(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    import_key_aes(
        global,
        format,
        key_data,
        extractable,
        usages,
        ALG_AES_CBC,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-export-key>
pub(crate) fn export_key_aes_cbc(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    export_key_aes(format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-get-key-length>
pub(crate) fn get_key_length_aes_cbc(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    get_key_length_aes(normalized_derived_key_algorithm)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-encrypt>
pub(crate) fn encrypt_aes_gcm(
    normalized_algorithm: &SubtleAesGcmParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If plaintext has a length greater than 2^39 - 256 bytes, then throw an OperationError.
    if plaintext.len() as u64 > (2 << 39) - 256 {
        return Err(Error::Operation(None));
    }

    // Step 2. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
    // then throw an OperationError.
    // NOTE: servo does not currently support 128-bit platforms, so this can never happen

    // Step 3. If the additionalData member of normalizedAlgorithm is present and has a length
    // greater than 2^64 - 1 bytes, then throw an OperationError.
    if normalized_algorithm
        .additional_data
        .as_ref()
        .is_some_and(|data| data.len() > u64::MAX as usize)
    {
        return Err(Error::Operation(None));
    }

    // Step 4.
    // If the tagLength member of normalizedAlgorithm is not present:
    //     Let tagLength be 128.
    // If the tagLength member of normalizedAlgorithm is one of 32, 64, 96, 104, 112, 120 or 128:
    //     Let tagLength be equal to the tagLength member of normalizedAlgorithm
    // Otherwise:
    //     throw an OperationError.
    let tag_length = match normalized_algorithm.tag_length {
        None => 128,
        Some(length) if matches!(length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => length,
        _ => {
            return Err(Error::Operation(None));
        },
    };

    // Step 5. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // an empty byte sequence otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 6. Let C and T be the outputs that result from performing the Authenticated Encryption
    // Function described in Section 7.1 of [NIST-SP800-38D] using AES as the block cipher, the
    // contents of the iv member of normalizedAlgorithm as the IV input parameter, the contents of
    // additionalData as the A input parameter, tagLength as the t pre-requisite and the contents
    // of plaintext as the input plaintext.
    let key_length = key.handle().as_bytes().len();
    let iv_length = normalized_algorithm.iv.len();
    let mut ciphertext = plaintext.to_vec();
    let key_bytes = key.handle().as_bytes();
    let tag = match (key_length, iv_length) {
        (16, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (16, 16) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm128Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (24, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (32, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes256Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (16, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (24, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (32, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes256Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        _ => {
            log::warn!(
                "Missing AES-GCM encryption implementation with {key_length}-byte key and {iv_length}-byte IV"
            );
            return Err(Error::NotSupported(None));
        },
    };

    // Step 7. Let ciphertext be equal to C | T, where '|' denotes concatenation.
    ciphertext.extend_from_slice(&tag.unwrap()[..tag_length as usize / 8]);

    // Step 8. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-decrypt>
pub(crate) fn decrypt_aes_gcm(
    normalized_algorithm: &SubtleAesGcmParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1.
    // If the tagLength member of normalizedAlgorithm is not present:
    //     Let tagLength be 128.
    // If the tagLength member of normalizedAlgorithm is one of 32, 64, 96, 104, 112, 120 or 128:
    //     Let tagLength be equal to the tagLength member of normalizedAlgorithm
    // Otherwise:
    //     throw an OperationError.
    let tag_length = match normalized_algorithm.tag_length {
        None => 128,
        Some(length) if matches!(length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => length as usize,
        _ => {
            return Err(Error::Operation(None));
        },
    };

    // Step 2. If ciphertext has a length in bits less than tagLength, then throw an
    // OperationError.
    if ciphertext.len() * 8 < tag_length {
        return Err(Error::Operation(None));
    }

    // Step 3. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
    // then throw an OperationError.
    // NOTE: servo does not currently support 128-bit platforms, so this can never happen

    // Step 4. If the additionalData member of normalizedAlgorithm is present and has a length
    // greater than 2^64 - 1 bytes, then throw an OperationError.
    // NOTE: servo does not currently support 128-bit platforms, so this can never happen

    // Step 5. Let tag be the last tagLength bits of ciphertext.
    // Step 6. Let actualCiphertext be the result of removing the last tagLength bits from
    // ciphertext.
    // NOTE: aes_gcm splits the ciphertext for us

    // Step 7. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // an empty byte sequence otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 8. Perform the Authenticated Decryption Function described in Section 7.2 of
    // [NIST-SP800-38D] using AES as the block cipher, the iv member of normalizedAlgorithm as the
    // IV input parameter, additionalData as the A input parameter, tagLength as the t
    // pre-requisite, actualCiphertext as the input ciphertext, C and tag as the authentication
    // tag, T.
    // If the result of the algorithm is the indication of inauthenticity, "FAIL":
    //     throw an OperationError
    // Otherwise:
    //     Let plaintext be the output P of the Authenticated Decryption Function.
    let mut plaintext = ciphertext.to_vec();
    let key_length = key.handle().as_bytes().len();
    let iv_length = normalized_algorithm.iv.len();
    let key_bytes = key.handle().as_bytes();
    let result = match (key_length, iv_length) {
        (16, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (16, 16) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm128Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (24, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (32, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes256Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (16, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (24, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (32, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes256Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        _ => {
            log::warn!(
                "Missing AES-GCM decryption implementation with {key_length}-byte key and {iv_length}-byte IV"
            );
            return Err(Error::NotSupported(None));
        },
    };
    if result.is_err() {
        return Err(Error::Operation(None));
    }

    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-generate-key>
pub(crate) fn generate_key_aes_gcm(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    generate_key_aes(
        global,
        normalized_algorithm,
        extractable,
        usages,
        ALG_AES_GCM,
        &[
            KeyUsage::Encrypt,
            KeyUsage::Decrypt,
            KeyUsage::WrapKey,
            KeyUsage::UnwrapKey,
        ],
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-import-key>
pub(crate) fn import_key_aes_gcm(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    import_key_aes(
        global,
        format,
        key_data,
        extractable,
        usages,
        ALG_AES_GCM,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-export-key>
pub(crate) fn export_key_aes_gcm(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    export_key_aes(format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-get-key-length>
pub(crate) fn get_key_length_aes_gcm(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    get_key_length_aes(normalized_derived_key_algorithm)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-wrap-key>
pub(crate) fn wrap_key_aes_kw(key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. If plaintext is not a multiple of 64 bits in length, then throw an OperationError.
    if plaintext.len() % 8 != 0 {
        return Err(Error::Operation(None));
    }

    // Step 2. Let ciphertext be the result of performing the Key Wrap operation described in
    // Section 2.2.1 of [RFC3394] with plaintext as the plaintext to be wrapped and using the
    // default Initial Value defined in Section 2.2.3.1 of the same document.
    let key_data = key.handle().as_bytes();
    let ciphertext = match key_data.len() {
        16 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes128::new(key_array);
            match kek.wrap_vec(plaintext) {
                Ok(key) => key,
                Err(_) => return Err(Error::Operation(None)),
            }
        },
        24 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes192::new(key_array);
            match kek.wrap_vec(plaintext) {
                Ok(key) => key,
                Err(_) => return Err(Error::Operation(None)),
            }
        },
        32 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes256::new(key_array);
            match kek.wrap_vec(plaintext) {
                Ok(key) => key,
                Err(_) => return Err(Error::Operation(None)),
            }
        },
        _ => return Err(Error::Operation(None)),
    };

    // Step 3. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-unwrap-key>
pub(crate) fn unwrap_key_aes_kw(key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. Let plaintext be the result of performing the Key Unwrap operation described in
    // Section 2.2.2 of [RFC3394] with ciphertext as the input ciphertext and using the default
    // Initial Value defined in Section 2.2.3.1 of the same document.
    // Step 2. If the Key Unwrap operation returns an error, then throw an OperationError.
    let key_data = key.handle().as_bytes();
    let plaintext = match key_data.len() {
        16 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes128::new(key_array);
            match kek.unwrap_vec(ciphertext) {
                Ok(key) => key,
                Err(_) => return Err(Error::Operation(None)),
            }
        },
        24 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes192::new(key_array);
            match kek.unwrap_vec(ciphertext) {
                Ok(key) => key,
                Err(_) => return Err(Error::Operation(None)),
            }
        },
        32 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes256::new(key_array);
            match kek.unwrap_vec(ciphertext) {
                Ok(key) => key,
                Err(_) => return Err(Error::Operation(None)),
            }
        },
        _ => return Err(Error::Operation(None)),
    };

    // Step 3. Return plaintext.
    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-generate-key>
pub(crate) fn generate_key_aes_kw(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    generate_key_aes(
        global,
        normalized_algorithm,
        extractable,
        usages,
        ALG_AES_KW,
        &[KeyUsage::WrapKey, KeyUsage::UnwrapKey],
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-import-key>
pub(crate) fn import_key_aes_kw(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    import_key_aes(
        global,
        format,
        key_data,
        extractable,
        usages,
        ALG_AES_KW,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-export-key>
pub(crate) fn export_key_aes_kw(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    export_key_aes(format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-get-key-length>
pub(crate) fn get_key_length_aes_kw(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    get_key_length_aes(normalized_derived_key_algorithm)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-generate-key>
#[allow(clippy::too_many_arguments)]
fn generate_key_aes(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    alg_name: &str,
    allowed_usages: &[KeyUsage],
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If usages contains any entry which is not one of allowed_usages, then throw a SyntaxError.
    if usages.iter().any(|usage| !allowed_usages.contains(usage)) {
        return Err(Error::Syntax(None));
    }

    // Step 2. If the length member of normalizedAlgorithm is not equal to one of 128, 192 or 256,
    // then throw an OperationError.
    if !matches!(normalized_algorithm.length, 128 | 192 | 256) {
        return Err(Error::Operation(None));
    }

    // Step 3. Generate an AES key of length equal to the length member of normalizedAlgorithm.
    // Step 4. If the key generation step fails, then throw an OperationError.
    let mut rand = vec![0; normalized_algorithm.length as usize / 8];
    if OsRng.try_fill_bytes(&mut rand).is_err() {
        return Err(Error::JSFailed);
    }

    let handle = match normalized_algorithm.length {
        128 => Handle::Aes128(rand),
        192 => Handle::Aes192(rand),
        256 => Handle::Aes256(rand),
        _ => return Err(Error::Operation(None)),
    };

    // Step 6. Let algorithm be a new AesKeyAlgorithm.
    // Step 7. Set the name attribute of algorithm to alg_name.
    // Step 8. Set the length attribute of algorithm to equal the length member of normalizedAlgorithm.
    let algorithm = SubtleAesKeyAlgorithm {
        name: alg_name.to_string(),
        length: normalized_algorithm.length,
    };

    // Step 5. Let key be a new CryptoKey object representing the generated AES key.
    // Step 9. Set the [[type]] internal slot of key to "secret".
    // Step 10. Set the [[algorithm]] internal slot of key to algorithm.
    // Step 11. Set the [[extractable]] internal slot of key to be extractable.
    // Step 12. Set the [[usages]] internal slot of key to be usages.
    let crypto_key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm),
        usages,
        handle,
        can_gc,
    );

    // Step 13. Return key.
    Ok(crypto_key)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-import-key>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-import-key>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-import-key>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-import-key>
fn import_key_aes(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    alg_name: &str,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If usages contains an entry which is not one of "encrypt", "decrypt", "wrapKey" or
    // "unwrapKey", then throw a SyntaxError.
    if usages.iter().any(|usage| {
        !matches!(
            usage,
            KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
        )
    }) || usages.is_empty()
    {
        return Err(Error::Syntax(None));
    }

    // Step 2.
    let data;
    match format {
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_secret => {
            // Step 2.1. Let data be keyData.
            data = key_data.to_vec();

            // Step 2.2. If the length in bits of data is not 128, 192 or 256 then throw a DataError.
            if !matches!(data.len() * 8, 128 | 192 | 256) {
                return Err(Error::Data(None));
            }
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. If keyData is a JsonWebKey dictionary: Let jwk equal keyData.
            // Otherwise: Throw a DataError.
            // NOTE: Deserialize keyData to JsonWebKey dictionary by calling JsonWebKey::parse
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 2.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(None));
            }

            // Step 2.3. If jwk does not meet the requirements of Section 6.4 of JSON Web
            // Algorithms [JWA], then throw a DataError.
            // NOTE: Done by Step 2.4 and 2.5.

            // Step 2.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = Base64UrlUnpadded::decode_vec(&jwk.k.as_ref().ok_or(Error::Data(None))?.str())
                .map_err(|_| Error::Data(None))?;

            // NOTE: This function is shared by AES-CBC, AES-CTR, AES-GCM and AES-KW.
            // Different static texts are used in different AES types, in the following step.
            let alg_matching = match alg_name {
                ALG_AES_CBC => ["A128CBC", "A192CBC", "A256CBC"],
                ALG_AES_CTR => ["A128CTR", "A192CTR", "A256CTR"],
                ALG_AES_GCM => ["A128GCM", "A192GCM", "A256GCM"],
                ALG_AES_KW => ["A128KW", "A192KW", "A256KW"],
                _ => unreachable!(),
            };

            // Step 2.5.
            match data.len() * 8 {
                // If the length in bits of data is 128:
                128 => {
                    // If the alg field of jwk is present, and is not "A128CBC", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A128CTR", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A128GCM", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A128KW", then throw a DataError.
                    // NOTE: Only perform the step of the corresponding AES type.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != alg_matching[0]) {
                        return Err(Error::Data(None));
                    }
                },
                // If the length in bits of data is 192:
                192 => {
                    // If the alg field of jwk is present, and is not "A192CBC", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A192CTR", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A192GCM", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A192KW", then throw a DataError.
                    // NOTE: Only perform the step of the corresponding AES type.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != alg_matching[1]) {
                        return Err(Error::Data(None));
                    }
                },
                // If the length in bits of data is 256:
                256 => {
                    // If the alg field of jwk is present, and is not "A256CBC", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A256CTR", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A256GCM", then throw a DataError.
                    // If the alg field of jwk is present, and is not "A256KW", then throw a DataError.
                    // NOTE: Only perform the step of the corresponding AES type.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != alg_matching[2]) {
                        return Err(Error::Data(None));
                    }
                },
                // Otherwise:
                _ => {
                    // throw a DataError.
                    return Err(Error::Data(None));
                },
            }

            // Step 2.6. If usages is non-empty and the use field of jwk is present and is not
            // "enc", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(None));
            }

            // Step 2.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(None));
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError
            return Err(Error::NotSupported(None));
        },
    };

    // Step 5. Let algorithm be a new AesKeyAlgorithm.
    // Step 6. Set the name attribute of algorithm to alg_name.
    // Step 7. Set the length attribute of algorithm to the length, in bits, of data.
    let algorithm = SubtleAesKeyAlgorithm {
        name: alg_name.to_string(),
        length: (data.len() * 8) as u16,
    };

    // Step 3. Let key be a new CryptoKey object representing an AES key with value data.
    // Step 4. Set the [[type]] internal slot of key to "secret".
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let handle = match data.len() * 8 {
        128 => Handle::Aes128(data.to_vec()),
        192 => Handle::Aes192(data.to_vec()),
        256 => Handle::Aes256(data.to_vec()),
        _ => {
            return Err(Error::Data(None));
        },
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm),
        usages,
        handle,
        can_gc,
    );

    // Step 9. Return key.
    Ok(key)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-export-key>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-export-key>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-export-key>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-export-key>
fn export_key_aes(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. If the underlying cryptographic key material represented by the [[handle]]
    // internal slot of key cannot be accessed, then throw an OperationError.
    // NOTE: key.handle() guarantees access.

    // Step 2.
    let result;
    match format {
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_secret => match key.handle() {
            // Step 2.1. Let data be a byte sequence containing the raw octets of the key
            // represented by the [[handle]] internal slot of key.
            // Step 2.2. Let result be data.
            Handle::Aes128(key_data) => {
                result = ExportedKey::Bytes(key_data.clone());
            },
            Handle::Aes192(key_data) => {
                result = ExportedKey::Bytes(key_data.clone());
            },
            Handle::Aes256(key_data) => {
                result = ExportedKey::Bytes(key_data.clone());
            },
            _ => unreachable!(),
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.3. Set the k attribute of jwk to be a string containing the raw octets of
            // the key represented by the [[handle]] internal slot of key, encoded according to
            // Section 6.4 of JSON Web Algorithms [JWA].
            let k = match key.handle() {
                Handle::Aes128(key) => Base64UrlUnpadded::encode_string(key),
                Handle::Aes192(key) => Base64UrlUnpadded::encode_string(key),
                Handle::Aes256(key) => Base64UrlUnpadded::encode_string(key),
                _ => unreachable!(),
            };

            // Step 2.4.
            // If the length attribute of key is 128: Set the alg attribute of jwk to the string "A128CTR".
            // If the length attribute of key is 192: Set the alg attribute of jwk to the string "A192CTR".
            // If the length attribute of key is 256: Set the alg attribute of jwk to the string "A256CTR".
            //
            // If the length attribute of key is 128: Set the alg attribute of jwk to the string "A128CTR".
            // If the length attribute of key is 192: Set the alg attribute of jwk to the string "A192CTR".
            // If the length attribute of key is 256: Set the alg attribute of jwk to the string "A256CTR".
            //
            // If the length attribute of key is 128: Set the alg attribute of jwk to the string "A128CTR".
            // If the length attribute of key is 192: Set the alg attribute of jwk to the string "A192CTR".
            // If the length attribute of key is 256: Set the alg attribute of jwk to the string "A256CTR".
            //
            // If the length attribute of key is 128: Set the alg attribute of jwk to the string "A128CTR".
            // If the length attribute of key is 192: Set the alg attribute of jwk to the string "A192CTR".
            // If the length attribute of key is 256: Set the alg attribute of jwk to the string "A256CTR".
            //
            // NOTE: Check key length via key.handle()
            let alg = match (key.handle(), key.algorithm().name()) {
                (Handle::Aes128(_), ALG_AES_CTR) => "A128CTR",
                (Handle::Aes192(_), ALG_AES_CTR) => "A192CTR",
                (Handle::Aes256(_), ALG_AES_CTR) => "A256CTR",
                (Handle::Aes128(_), ALG_AES_CBC) => "A128CBC",
                (Handle::Aes192(_), ALG_AES_CBC) => "A192CBC",
                (Handle::Aes256(_), ALG_AES_CBC) => "A256CBC",
                (Handle::Aes128(_), ALG_AES_GCM) => "A128GCM",
                (Handle::Aes192(_), ALG_AES_GCM) => "A192GCM",
                (Handle::Aes256(_), ALG_AES_GCM) => "A256GCM",
                (Handle::Aes128(_), ALG_AES_KW) => "A128KW",
                (Handle::Aes192(_), ALG_AES_KW) => "A192KW",
                (Handle::Aes256(_), ALG_AES_KW) => "A256KW",
                _ => unreachable!(),
            };

            // Step 2.5. Set the key_ops attribute of jwk to equal the [[usages]] internal slot of key.
            let key_ops = key
                .usages()
                .iter()
                .map(|usage| DOMString::from(usage.as_str()))
                .collect::<Vec<DOMString>>();

            // Step 2.1. Let jwk be a new JsonWebKey dictionary.
            // Step 2.2. Set the kty attribute of jwk to the string "oct".
            // Step 2.6. Set the ext attribute of jwk to equal the [[extractable]] internal slot of key.
            let jwk = JsonWebKey {
                kty: Some(DOMString::from("oct")),
                k: Some(DOMString::from(k)),
                alg: Some(DOMString::from(alg)),
                key_ops: Some(key_ops),
                ext: Some(key.Extractable()),
                ..Default::default()
            };

            // Step 2.7. Let result be jwk.
            result = ExportedKey::Jwk(Box::new(jwk));
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(None));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-get-key-length>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-get-key-length>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-get-key-length>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-get-key-length>
pub(crate) fn get_key_length_aes(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    // Step 1. If the length member of normalizedDerivedKeyAlgorithm is not 128, 192 or 256, then
    // throw a OperationError.
    if !matches!(normalized_derived_key_algorithm.length, 128 | 192 | 256) {
        return Err(Error::Operation(None));
    }

    // Step 2. Return the length member of normalizedDerivedKeyAlgorithm.
    Ok(Some(normalized_derived_key_algorithm.length as u32))
}
