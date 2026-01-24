/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use pkcs8::PrivateKeyInfo;
use pkcs8::der::asn1::{BitStringRef, OctetString, OctetStringRef};
use pkcs8::der::{AnyRef, Decode, Encode};
use pkcs8::spki::{AlgorithmIdentifier, ObjectIdentifier, SubjectPublicKeyInfo};
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_X25519, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    SubtleEcdhKeyDeriveParams, SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

/// `id-X25519` object identifier defined in [RFC8410]
const X25519_OID_STRING: &str = "1.3.101.110";

const PRIVATE_KEY_LENGTH: usize = 32;
const PUBLIC_KEY_LENGTH: usize = 32;

/// <https://w3c.github.io/webcrypto/#x25519-operations-derive-bits>
pub(crate) fn derive_bits(
    normalized_algorithm: &SubtleEcdhKeyDeriveParams,
    key: &CryptoKey,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(None));
    }

    // Step 2. Let publicKey be the public member of normalizedAlgorithm.
    let public_key = normalized_algorithm.public.root();

    // Step 3. If the [[type]] internal slot of publicKey is not "public", then throw an
    // InvalidAccessError.
    if public_key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(None));
    }

    // Step 4. If the name attribute of the [[algorithm]] internal slot of publicKey is not equal
    // to the name property of the [[algorithm]] internal slot of key, then throw an
    // InvalidAccessError.
    if public_key.algorithm().name() != key.algorithm().name() {
        return Err(Error::InvalidAccess(None));
    }

    // Step 5. Let secret be the result of performing the X25519 function specified in [RFC7748]
    // Section 5 with key as the X25519 private key k and the X25519 public key represented by the
    // [[handle]] internal slot of publicKey as the X25519 public key u.
    let Handle::X25519PrivateKey(private_key) = key.handle() else {
        return Err(Error::Operation(None));
    };
    let Handle::X25519PublicKey(public_key) = public_key.handle() else {
        return Err(Error::Operation(None));
    };
    let shared_key = private_key.diffie_hellman(public_key);
    let secret = shared_key.as_bytes();

    // Step 6. If secret is the all-zero value, then throw a OperationError. This check must be
    // performed in constant-time, as per [RFC7748] Section 6.1.
    let mut is_all_zero = true;
    for byte in secret {
        is_all_zero &= *byte == 0;
    }
    if is_all_zero {
        return Err(Error::Operation(None));
    }

    // Step 7.
    // If length is null:
    //     Return secret
    // Otherwise:
    //     If the length of secret in bits is less than length:
    //         throw an OperationError.
    //     Otherwise:
    //         Return a byte sequence containing the first length bits of secret.
    match length {
        None => Ok(secret.to_vec()),
        Some(length) => {
            if secret.len() * 8 < length as usize {
                Err(Error::Operation(None))
            } else {
                let mut secret = secret[..length.div_ceil(8) as usize].to_vec();
                if length % 8 != 0 {
                    // Clean excess bits in last byte of secret.
                    let mask = u8::MAX << (8 - length % 8);
                    if let Some(last_byte) = secret.last_mut() {
                        *last_byte &= mask;
                    }
                }
                Ok(secret)
            }
        },
    }
}

/// <https://w3c.github.io/webcrypto/#x25519-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains an entry which is not "deriveKey" or "deriveBits" then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
    {
        return Err(Error::Syntax(None));
    }

    // Step 2. Generate an X25519 key pair, with the private key being 32 random bytes, and the
    // public key being X25519(a, 9), as defined in [RFC7748], section 6.1.
    let private_key = StaticSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&private_key);

    // Step 3. Let algorithm be a new KeyAlgorithm object.
    // Step 4. Set the name attribute of algorithm to "X25519".
    let algorithm = SubtleKeyAlgorithm {
        name: ALG_X25519.to_string(),
    };

    // Step 5. Let publicKey be a new CryptoKey representing the public key of the generated key pair.
    // Step 6. Set the [[type]] internal slot of publicKey to "public"
    // Step 7. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 8. Set the [[extractable]] internal slot of publicKey to true.
    // Step 9. Set the [[usages]] internal slot of publicKey to be the empty list.
    let public_key = CryptoKey::new(
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
        Vec::new(),
        Handle::X25519PublicKey(public_key),
        can_gc,
    );

    // Step 10. Let privateKey be a new CryptoKey representing the private key of the generated key pair.
    // Step 11. Set the [[type]] internal slot of privateKey to "private"
    // Step 12. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 13. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 14. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "deriveKey", "deriveBits" ].
    let private_key = CryptoKey::new(
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages
            .iter()
            .filter(|usage| matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            .cloned()
            .collect(),
        Handle::X25519PrivateKey(private_key),
        can_gc,
    );

    // Step 15. Let result be a new CryptoKeyPair dictionary.
    // Step 16. Set the publicKey attribute of result to be publicKey.
    // Step 17. Set the privateKey attribute of result to be privateKey.
    let result = CryptoKeyPair {
        publicKey: Some(public_key),
        privateKey: Some(private_key),
    };

    // Step 18. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#x25519-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let key = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If usages is not empty then throw a SyntaxError.
            if !usages.is_empty() {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let spki = SubjectPublicKeyInfo::<AnyRef, BitStringRef>::from_der(key_data)
                .map_err(|_| Error::Data(None))?;

            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the id-X25519 object identifier
            // defined in [RFC8410], then throw a DataError.
            if spki.algorithm.oid != ObjectIdentifier::new_unwrap(X25519_OID_STRING) {
                return Err(Error::Data(None));
            }

            // Step 2.5. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            if spki.algorithm.parameters.is_some() {
                return Err(Error::Data(None));
            }

            // Step 2.6. Let publicKey be the X25519 public key identified by the subjectPublicKey
            // field of spki.
            let key_bytes: [u8; PUBLIC_KEY_LENGTH] = spki
                .subject_public_key
                .as_bytes()
                .ok_or(Error::Data(None))?
                .try_into()
                .map_err(|_| Error::Data(None))?;
            let public_key = PublicKey::from(key_bytes);

            // Step 2.7. Let key be a new CryptoKey that represents publicKey.
            // Step 2.8. Set the [[type]] internal slot of key to "public"
            // Step 2.9. Let algorithm be a new KeyAlgorithm.
            // Step 2.10. Set the name attribute of algorithm to "X25519".
            // Step 2.11. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_X25519.to_string(),
            };
            CryptoKey::new(
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::X25519PublicKey(public_key),
                can_gc,
            )
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If usages contains an entry which is not "deriveKey" or "deriveBits" then
            // throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
            let private_key_info =
                PrivateKeyInfo::from_der(key_data).map_err(|_| Error::Data(None))?;

            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the id-X25519 object
            // identifier defined in [RFC8410], then throw a DataError.
            if private_key_info.algorithm.oid != ObjectIdentifier::new_unwrap(X25519_OID_STRING) {
                return Err(Error::Data(None));
            }

            // Step 2.5. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            if private_key_info.algorithm.parameters.is_some() {
                return Err(Error::Data(None));
            }

            // Step 2.6. Let curvePrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as the ASN.1 CurvePrivateKey structure specified in Section 7 of [RFC8410], and
            // exactData set to true.
            // Step 2.7. If an error occurred while parsing, then throw a DataError.
            let curve_private_key = OctetStringRef::from_der(private_key_info.private_key)
                .map_err(|_| Error::Data(None))?;
            let key_bytes: [u8; PRIVATE_KEY_LENGTH] = curve_private_key
                .as_bytes()
                .try_into()
                .map_err(|_| Error::Data(None))?;
            let private_key = StaticSecret::from(key_bytes);

            // Step 2.8. Let key be a new CryptoKey that represents the X25519 private key
            // identified by curvePrivateKey.
            // Step 2.9. Set the [[type]] internal slot of key to "private"
            // Step 2.10. Let algorithm be a new KeyAlgorithm.
            // Step 2.11. Set the name attribute of algorithm to "X25519".
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_X25519.to_string(),
            };
            CryptoKey::new(
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::X25519PrivateKey(private_key),
                can_gc,
            )
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 2.2. If the d field is present and if usages contains an entry which is not
            // "deriveKey" or "deriveBits" then throw a SyntaxError.
            if jwk.d.is_some() &&
                usages
                    .iter()
                    .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. If the d field is not present and if usages is not empty then throw a
            // SyntaxError.
            if jwk.d.is_none() && !usages.is_empty() {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. If the kty field of jwk is not "OKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "OKP") {
                return Err(Error::Data(None));
            }

            // Step 2.2. If the crv field of jwk is not "X25519", then throw a DataError.
            if jwk.crv.as_ref().is_none_or(|crv| crv != ALG_X25519) {
                return Err(Error::Data(None));
            }

            // Step 2.2. If usages is non-empty and the use field of jwk is present and is not
            // equal to "enc" then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(None));
            }

            // Step 2.2. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.2. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(None));
            }

            // Step 2.9.
            // If the d field is present:
            let (handle, key_type) = if jwk.d.is_some() {
                // Step 2.9.1. If jwk does not meet the requirements of the JWK private key format
                // described in Section 2 of [RFC8037], then throw a DataError.
                let x = jwk.decode_required_string_field(JwkStringField::X)?;
                let d = jwk.decode_required_string_field(JwkStringField::D)?;
                let public_key_bytes: [u8; PUBLIC_KEY_LENGTH] =
                    x.try_into().map_err(|_| Error::Data(None))?;
                let private_key_bytes: [u8; PRIVATE_KEY_LENGTH] =
                    d.try_into().map_err(|_| Error::Data(None))?;
                let public_key = PublicKey::from(public_key_bytes);
                let private_key = StaticSecret::from(private_key_bytes);
                if PublicKey::from(&private_key) != public_key {
                    return Err(Error::Data(None));
                }

                // Step 2.9.1. Let key be a new CryptoKey object that represents the X25519 private
                // key identified by interpreting jwk according to Section 2 of [RFC8037].
                // NOTE: CryptoKey is created in Step 2.10 - 2.12.
                let handle = Handle::X25519PrivateKey(private_key);

                // Step 2.9.1. Set the [[type]] internal slot of Key to "private".
                let key_type = KeyType::Private;

                (handle, key_type)
            }
            // Otherwise:
            else {
                // Step 2.9.1. If jwk does not meet the requirements of the JWK public key format
                // described in Section 2 of [RFC8037], then throw a DataError.
                let x = jwk.decode_required_string_field(JwkStringField::X)?;
                let public_key_bytes: [u8; PUBLIC_KEY_LENGTH] =
                    x.try_into().map_err(|_| Error::Data(None))?;
                let public_key = PublicKey::from(public_key_bytes);

                // Step 2.9.1. Let key be a new CryptoKey object that represents the X25519 public
                // key identified by interpreting jwk according to Section 2 of [RFC8037].
                // NOTE: CryptoKey is created in Step 2.10 - 2.12.
                let handle = Handle::X25519PublicKey(public_key);

                // Step 2.9.1. Set the [[type]] internal slot of Key to "public".
                let key_type = KeyType::Public;

                (handle, key_type)
            };

            // Step 2.10. Let algorithm be a new instance of a KeyAlgorithm object.
            // Step 2.11. Set the name attribute of algorithm to "X25519".
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_X25519.to_string(),
            };
            CryptoKey::new(
                global,
                key_type,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                handle,
                can_gc,
            )
        },
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_public => {
            // Step 2.1. If usages is not empty then throw a SyntaxError.
            if !usages.is_empty() {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. If the length in bits of keyData is not 256 then throw a DataError.
            if key_data.len() != 32 {
                return Err(Error::Data(None));
            }

            // Step 2.3. Let algorithm be a new KeyAlgorithm object.
            // Step 2.4. Set the name attribute of algorithm to "X25519".
            // Step 2.5. Let key be a new CryptoKey representing the key data provided in keyData.
            // Step 2.6. Set the [[type]] internal slot of key to "public"
            // Step 2.7. Set the [[algorithm]] internal slot of key to algorithm.
            let key_bytes: [u8; PUBLIC_KEY_LENGTH] =
                key_data.try_into().map_err(|_| Error::Data(None))?;
            let public_key = PublicKey::from(key_bytes);
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_X25519.to_string(),
            };
            CryptoKey::new(
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::X25519PublicKey(public_key),
                can_gc,
            )
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(None));
        },
    };

    // Step 3. Return key
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#x25519-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the CryptoKey to be exported.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    // NOTE: Done in Step 3.

    // Step 3.
    let result = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //     * Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the
            //       following properties:
            //         * Set the algorithm object identifier to the id-X25519 OID defined in
            //           [RFC8410].
            //     * Set the subjectPublicKey field to keyData.
            let Handle::X25519PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(None));
            };
            let data = SubjectPublicKeyInfo::<BitStringRef, _> {
                algorithm: AlgorithmIdentifier {
                    oid: ObjectIdentifier::new_unwrap(X25519_OID_STRING),
                    parameters: None,
                },
                subject_public_key: BitStringRef::from_bytes(public_key.as_bytes())
                    .map_err(|_| Error::Data(None))?,
            };

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.to_der().map_err(|_| Error::Operation(None))?)
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //     * Set the version field to 0.
            //     * Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1
            //       type with the following properties:
            //         * Set the algorithm object identifier to the id-X25519 OID defined in
            //         [RFC8410].
            //     * Set the privateKey field to the result of DER-encoding a CurvePrivateKey ASN.1
            //       type, as defined in Section 7 of [RFC8410], that represents the X25519 private
            //       key represented by the [[handle]] internal slot of key
            let Handle::X25519PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(None));
            };
            let curve_private_key =
                OctetString::new(private_key.as_bytes()).map_err(|_| Error::Data(None))?;
            let data = PrivateKeyInfo {
                algorithm: AlgorithmIdentifier {
                    oid: ObjectIdentifier::new_unwrap(X25519_OID_STRING),
                    parameters: None,
                },
                private_key: &curve_private_key.to_der().map_err(|_| Error::Data(None))?,
                public_key: None,
            };

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.to_der().map_err(|_| Error::Operation(None))?)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            // Step 3.2. Set the kty attribute of jwk to "OKP".
            // Step 3.3. Set the crv attribute of jwk to "X25519".
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("OKP")),
                crv: Some(DOMString::from(ALG_X25519)),
                ..Default::default()
            };

            // Step 3.4. Set the x attribute of jwk according to the definition in Section 2 of
            // [RFC8037].
            match key.handle() {
                Handle::X25519PrivateKey(private_key) => {
                    let public_key = PublicKey::from(private_key);
                    jwk.encode_string_field(JwkStringField::X, public_key.as_bytes());
                },
                Handle::X25519PublicKey(public_key) => {
                    jwk.encode_string_field(JwkStringField::X, public_key.as_bytes());
                },
                _ => return Err(Error::Operation(None)),
            }

            // Step 3.5.
            // If the [[type]] internal slot of key is "private"
            //     Set the d attribute of jwk according to the definition in Section 2 of
            //     [RFC8037].
            if key.Type() == KeyType::Private {
                if let Handle::X25519PrivateKey(private_key) = key.handle() {
                    jwk.encode_string_field(JwkStringField::D, private_key.as_bytes());
                } else {
                    return Err(Error::Operation(None));
                }
            }

            // Step 3.6. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 3.7. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.8. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_public => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2. Let data be a byte sequence representing the X25519 public key represented
            // by the [[handle]] internal slot of key.
            let Handle::X25519PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(None));
            };
            let data = public_key.as_bytes();

            // Step 3.3. Let result be data.
            ExportedKey::Bytes(data.to_vec())
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(None));
        },
    };

    // Step 4. Return result.
    Ok(result)
}
