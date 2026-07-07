/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use elliptic_curve::ctutils::CtEq;
use js::context::JSContext;
use pkcs8::der::asn1::OctetStringRef;
use pkcs8::der::{Decode, Encode};
use pkcs8::{AlgorithmIdentifierRef, ObjectIdentifier, PrivateKeyInfoRef, SubjectPublicKeyInfoRef};
use x448::{PublicKey, StaticSecret};
use zeroize::Zeroizing;

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
    CryptoAlgorithm, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    SubtleEcdhKeyDeriveParams, SubtleKeyAlgorithm,
};

/// `id-X448` object identifier defined in [RFC8410]
const X448_OID_STRING: &str = "1.3.101.111";

const PRIVATE_KEY_LENGTH: usize = 56;
pub(crate) const SECRET_LENGTH: usize = 56;

/// <https://wicg.github.io/webcrypto-secure-curves/#x448>
pub(crate) fn derive_bits(
    normalized_algorithm: &SubtleEcdhKeyDeriveParams,
    key: &CryptoKey,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"private\"".into(),
        )));
    }

    // Step 2. Let publicKey be the public member of normalizedAlgorithm.
    let public_key = normalized_algorithm.public.root();

    // Step 3. If the [[type]] internal slot of publicKey is not "public", then throw an
    // InvalidAccessError.
    if public_key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of publicKey is not \"public\"".into(),
        )));
    }

    // Step 4. If the name attribute of the [[algorithm]] internal slot of publicKey is not equal to
    // the name property of the [[algorithm]] internal slot of key, then throw an
    // InvalidAccessError.
    if public_key.algorithm().name() != key.algorithm().name() {
        return Err(Error::InvalidAccess(Some(
            "[[algorithm]] internal slot of publicKey does not match \
                [[algorithm]] internal slot of key"
                .into(),
        )));
    }

    // Step 5. Let secret be the result of performing the X448 function specified in [RFC7748]
    // Section 5 with key as the X448 private key k and the X448 public key represented by the
    // [[handle]] internal slot of publicKey as the X448 public key u.
    let Handle::X448PrivateKey(private_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an X448 private key".into(),
        )));
    };
    let Handle::X448PublicKey(public_key) = public_key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of publicKey is not an X448 public key".into(),
        )));
    };
    let secret = private_key.diffie_hellman(public_key);

    // Step 6. If secret is the all-zero value, then throw a OperationError. This check must be
    // performed in constant-time, as per [RFC7748] Section 6.2.
    if secret.as_bytes().ct_eq(&[0u8; SECRET_LENGTH]).into() {
        return Err(Error::Operation(Some(
            "The secret is the all-zero value".into(),
        )));
    }

    // Step 7.
    // If length is null:
    //     Return secret
    // Otherwise:
    //     If the length of secret in bits is less than length:
    //         throw an OperationError.
    //     Otherwise:
    //         Return an octet string containing the first length bits of secret.
    let secret_slice = secret.as_bytes();
    match length {
        None => Ok(secret_slice.to_vec()),
        Some(length) => {
            if secret_slice.len() * 8 < length as usize {
                Err(Error::Operation(None))
            } else {
                let mut secret = secret_slice[..length.div_ceil(8) as usize].to_vec();
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

/// <https://wicg.github.io/webcrypto-secure-curves/#x448-description>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains an entry which is not "deriveKey" or "deriveBits" then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
    {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"deriveKey\" or \"deriveBits\"".into(),
        )));
    }

    // Step 2. Generate an X448 key pair, with the private key being 56 random bytes, and the public
    // key being X448(a, 5), as defined in [RFC7748], section 6.2.
    let mut rng = rand::rng();
    let private_key = StaticSecret::random_from_rng(&mut rng);
    let public_key = PublicKey::from(&private_key);

    // Step 3. Let algorithm be a new KeyAlgorithm object.
    // Step 4. Set the name attribute of algorithm to "X448".
    let algorithm = SubtleKeyAlgorithm {
        name: CryptoAlgorithm::X448,
    };

    // Step 5. Let publicKey be a new CryptoKey associated with the relevant global object of this
    // [HTML], and representing the public key of the generated key pair.
    // Step 6. Set the [[type]] internal slot of publicKey to "public"
    // Step 7. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 8. Set the [[extractable]] internal slot of publicKey to true.
    // Step 9. Set the [[usages]] internal slot of publicKey to be the empty list.
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
        Vec::new(),
        Handle::X448PublicKey(public_key),
    );

    // Step 10. Let privateKey be a new CryptoKey associated with the relevant global object of this
    // [HTML], and representing the private key of the generated key pair.
    // Step 11. Set the [[type]] internal slot of privateKey to "private"
    // Step 12. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 13. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 14. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "deriveKey", "deriveBits" ].
    let private_key = CryptoKey::new(
        cx,
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages
            .iter()
            .filter(|usage| matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            .cloned()
            .collect(),
        Handle::X448PrivateKey(private_key),
    );

    // Step 15. Let result be a new CryptoKeyPair dictionary.
    // Step 16. Set the publicKey attribute of result to be publicKey.
    // Step 17. Set the privateKey attribute of result to be privateKey.
    let result = CryptoKeyPair {
        publicKey: Some(public_key),
        privateKey: Some(private_key),
    };

    // Step 18. Return the result of converting result to an ECMAScript Object, as defined by
    // [WebIDL].
    // NOTE: The conversion of result to an ECMAScript Object is done in SubtleCrypto::Generate.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-secure-curves/#x448-description>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let key = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If usages is not empty then throw a SyntaxError.
            if !usages.is_empty() {
                return Err(Error::Syntax(Some("Usages is not empty".into())));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let spki = SubjectPublicKeyInfoRef::from_der(key_data).map_err(|_| {
                Error::Data(Some(
                    "Failed to parse the X448 public key in SPKI format".into(),
                ))
            })?;

            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the id-X448 object identifier
            // defined in [RFC8410], then throw a DataError.
            if spki.algorithm.oid != ObjectIdentifier::new_unwrap(X448_OID_STRING) {
                return Err(Error::Data(Some(
                    "The algorithm object identifier field of the algorithm field of spki \
                        is not equal to the id-X448 object identifier"
                        .into(),
                )));
            }

            // Step 2.5. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            if spki.algorithm.parameters.is_some() {
                return Err(Error::Data(Some(
                    "The parameters field of the algorithm field of spki is present".into(),
                )));
            }

            // Step 2.6. Let publicKey be the X448 public key identified by the subjectPublicKey
            // field of spki.
            let key_bytes = spki.subject_public_key.as_bytes().ok_or(Error::Data(Some(
                "The subjectPublicKey field in spki is not octet aligned".into(),
            )))?;
            let public_key = PublicKey::from_bytes_unchecked(key_bytes).ok_or(Error::Data(
                Some("The length of the subjectPublicKey in spki is not 56 bytes".into()),
            ))?;

            // Step 2.7. Let key be a new CryptoKey associated with the relevant global object of
            // this [HTML], and that represents publicKey.
            // Step 2.8. Set the [[type]] internal slot of key to "public"
            // Step 2.9. Let algorithm be a new KeyAlgorithm.
            // Step 2.10. Set the name attribute of algorithm to "X448".
            // Step 2.11. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::X448,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::X448PublicKey(public_key),
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
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"deriveKey\" or \"deriveBits\"".into(),
                )));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
            let private_key_info = PrivateKeyInfoRef::from_der(key_data).map_err(|_| {
                Error::Data(Some(
                    "Failed to parse the X448 private key to PKCS#8 document".into(),
                ))
            })?;

            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the id-X448 object
            // identifier defined in [RFC8410], then throw a DataError.
            if private_key_info.algorithm.oid != ObjectIdentifier::new_unwrap(X448_OID_STRING) {
                return Err(Error::Data(Some(
                    "The algorithm object identifier field of the privateKeyAlgorithm field of \
                        privateKeyInfo is not equal to the id-X448 object identifier"
                        .into(),
                )));
            }

            // Step 2.5. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            if private_key_info.algorithm.parameters.is_some() {
                return Err(Error::Data(Some(
                    "The parameters field of the privateKeyAlgorithm field of privateKeyInfo \
                        is present"
                        .into(),
                )));
            }

            // Step 2.6. Let curvePrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as the ASN.1 CurvePrivateKey structure specified in Section 7 of [RFC8410], and
            // exactData set to true.
            // Step 7. If an error occurred while parsing, then throw a DataError.
            let curve_private_key = private_key_info
                .private_key
                .decode_into::<&OctetStringRef>()
                .map_err(|_| {
                    Error::Data(Some(
                        "Failed to decode the privateKey field of PrivateKeyInfo ASN.1 structure"
                            .into(),
                    ))
                })?;
            let key_bytes: [u8; PRIVATE_KEY_LENGTH] =
                curve_private_key.as_bytes().try_into().map_err(|_| {
                    Error::Data(Some(
                        "Failed to extract the raw bytes from the CurvePrivateKey ASN.1 structure"
                            .into(),
                    ))
                })?;
            let curve_private_key = StaticSecret::from(key_bytes);

            // Step 2.8. Let key be a new CryptoKey associated with the relevant global object of
            // this [HTML], and that represents the X448 private key identified by curvePrivateKey.
            // Step 2.9. Set the [[type]] internal slot of key to "private"
            // Step 2.10. Let algorithm be a new KeyAlgorithm.
            // Step 2.11. Set the name attribute of algorithm to "X448".
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::X448,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::X448PrivateKey(curve_private_key),
            )
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(cx, key_data)?;

            // Step 2.2. If the d field is present and if usages contains an entry which is not
            // "deriveKey" or "deriveBits" then throw a SyntaxError.
            if jwk.d.is_some() &&
                usages
                    .iter()
                    .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            {
                return Err(Error::Syntax(Some(
                    "The d field is present and if usages contains an entry which is not \
                        \"deriveKey\" or \"deriveBits\""
                        .into(),
                )));
            }

            // Step 2.3. If the d field is not present and if usages is not empty then throw a
            // SyntaxError.
            if jwk.d.is_none() && !usages.is_empty() {
                return Err(Error::Syntax(Some(
                    "The d field is not present and if usages is not empty".into(),
                )));
            }

            // Step 2.4. If the kty field of jwk is not "OKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "OKP") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"OKP\"".into(),
                )));
            }

            // Step 2.5. If the crv field of jwk is not "X448", then throw a DataError.
            if jwk.crv.as_ref().is_none_or(|crv| crv != "X448") {
                return Err(Error::Data(Some(
                    "The crv field of jwk is not \"X448\"".into(),
                )));
            }

            // Step 2.6. If usages is non-empty and the use field of jwk is present and is not equal
            // to "enc" then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not equal to \
                        \"enc\""
                        .into(),
                )));
            }

            // Step 2.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some(
                    "the ext field of jwk is present and has the value false \
                        and extractable is true"
                        .into(),
                )));
            }

            // Step 2.9.
            // If the d field is present:
            let (handle, key_type) = if jwk.d.is_some() {
                // Step 2.9.1. If jwk does not meet the requirements of the JWK private key format
                // described in Section 2 of [RFC8037], then throw a DataError.
                let d = jwk.decode_required_string_field(JwkStringField::D)?;
                let x = jwk.decode_required_string_field(JwkStringField::X)?;
                let private_key_bytes: [u8; PRIVATE_KEY_LENGTH] =
                    d.as_slice().try_into().map_err(|_| {
                        Error::Data(Some("Invalid length of private key in 'd' field".into()))
                    })?;
                let public_key_bytes = x.as_slice();
                let private_key = StaticSecret::from(private_key_bytes);
                let public_key = PublicKey::from_bytes_unchecked(public_key_bytes).ok_or(
                    Error::Data(Some("Invalid length of private key in 'x' field".into())),
                )?;
                if PublicKey::from(&private_key) != public_key {
                    return Err(Error::Data(Some(
                        "Public key in 'x' field does not match private key in 'd' field".into(),
                    )));
                }

                // Step 2.9.2. Let key be a new CryptoKey object that represents the X448 private
                // key identified by interpreting jwk according to Section 2 of [RFC8037].
                // NOTE: CryptoKey is created in Step 2.10 - 2.12.
                let handle = Handle::X448PrivateKey(private_key);

                // Step 2.9.3. Set the [[type]] internal slot of Key to "private".
                let key_type = KeyType::Private;

                (handle, key_type)
            }
            // Otherwise:
            else {
                // Step 2.9.1. If jwk does not meet the requirements of the JWK public key format
                // described in Section 2 of [RFC8037], then throw a DataError.
                let x = jwk.decode_required_string_field(JwkStringField::X)?;
                let public_key_bytes = x.as_slice();
                let public_key = PublicKey::from_bytes_unchecked(public_key_bytes).ok_or(
                    Error::Data(Some("Invalid length of private key in 'x' field".into())),
                )?;

                // Step 2.9.2. Let key be a new CryptoKey object that represents the X448 public key
                // identified by interpreting jwk according to Section 2 of [RFC8037].
                // NOTE: CryptoKey is created in Step 2.10 - 2.12.
                let handle = Handle::X448PublicKey(public_key);

                // Step 2.9.3. Set the [[type]] internal slot of Key to "public".
                let key_type = KeyType::Public;

                (handle, key_type)
            };

            // Step 2.10. Let algorithm be a new instance of a KeyAlgorithm object.
            // Step 2.11. Set the name attribute of algorithm to "X448".
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::X448,
            };
            CryptoKey::new(
                cx,
                global,
                key_type,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                handle,
            )
        },
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_public => {
            // Step 2.1. If usages is not empty then throw a SyntaxError.
            if !usages.is_empty() {
                return Err(Error::Syntax(Some("Usages is not empty".into())));
            }

            // Step 2.2. Let data be keyData.
            let data = key_data;

            // Step 2.3. If the length in bits of data is not 448 then throw a DataError.
            if data.len() != 56 {
                return Err(Error::Data(Some("The key length is not 448 bits".into())));
            }

            // Step 2.4. Let algorithm be a new KeyAlgorithm object.
            // Step 2.5. Set the name attribute of algorithm to "X448".
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::X448,
            };

            // Step 2.6. Let key be a new CryptoKey associated with the relevant global object of
            // this [HTML], and that represents data.
            // Step 2.7. Set the [[type]] internal slot of key to "public"
            // Step 2.8. Set the [[algorithm]] internal slot of key to algorithm.
            let public_key = PublicKey::from_bytes_unchecked(data).ok_or(Error::Data(Some(
                "Failed to import public key from raw bytes".into(),
            )))?;
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::X448PublicKey(public_key),
            )
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported import key format for X448".into(),
            )));
        },
    };

    // Step 3. Return key
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-secure-curves/#x448>
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
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".into(),
                )));
            }

            // Step 3.2. Let data be an instance of the subjectPublicKeyInfo ASN.1 structure defined
            // in [RFC5280] with the following properties:
            // * Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //   properties:
            //     * Set the algorithm object identifier to the id-X448 OID defined in [RFC8410].
            // * Set the subjectPublicKey field to keyData.
            let Handle::X448PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "[[handle]] internal slot of key is not an X448 public key".into(),
                )));
            };
            let data = SubjectPublicKeyInfoRef {
                algorithm: AlgorithmIdentifierRef {
                    oid: ObjectIdentifier::new_unwrap(X448_OID_STRING),
                    parameters: None,
                },
                subject_public_key: public_key.as_bytes().try_into().map_err(|_| {
                    Error::Data(Some(
                        "Failed to construct the subjectPublicKey field of subjectPublicKeyInfo \
                            ASN.1 structure"
                            .into(),
                    ))
                })?,
            };

            // Step 3.3. Let result be a new ArrayBuffer associated with the relevant global object
            // of this [HTML], and containing data.
            // NOTE: The conversion to a new ArrayBuffer is done in SubtleCrypto::ExportKey.
            ExportedKey::new_bytes(data.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode the subjectPublicKeyInfo ASN.1 structure in DER-encoding"
                        .into(),
                ))
            })?)
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".into(),
                )));
            }

            // Step 3.2. Let data be an instance of the privateKeyInfo ASN.1 structure defined in
            // [RFC5208] with the following properties:
            // * Set the version field to 0.
            // * Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //   with the following properties:
            //     * Set the algorithm object identifier to the id-X448 OID defined in [RFC8410].
            // * Set the privateKey field to the result of DER-encoding a CurvePrivateKey ASN.1
            //   type, as defined in Section 7 of [RFC8410], that represents the X448 private key
            //   represented by the [[handle]] internal slot of key
            let Handle::X448PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "[[handle]] internal slot of key is not an X448 private key".into(),
                )));
            };
            let curve_private_key = OctetStringRef::new(private_key.as_bytes()).map_err(|_| {
                Error::Operation(Some(
                    "Failed to construct CurvePrivateKey ASN.1 structure".into(),
                ))
            })?;
            let encoded_curve_private_key: Zeroizing<Vec<u8>> = curve_private_key
                .to_der()
                .map_err(|_| {
                    Error::Operation(Some(
                        "Failed to encode CurvePrivateKey ASN.1 structure in DER-encoding".into(),
                    ))
                })?
                .into();
            let private_key_field =
                OctetStringRef::new(&encoded_curve_private_key).map_err(|_| {
                    Error::Operation(Some(
                        "Failed to construct privateKey field of privateKeyInfo ASN.1 structure"
                            .into(),
                    ))
                })?;
            let data = PrivateKeyInfoRef {
                algorithm: AlgorithmIdentifierRef {
                    oid: ObjectIdentifier::new_unwrap(X448_OID_STRING),
                    parameters: None,
                },
                private_key: private_key_field,
                public_key: None,
            };

            // Step 3.3. Let result be a new ArrayBuffer associated with the relevant global object
            // of this [HTML], and containing data.
            // NOTE: The conversion to a new ArrayBuffer is done in SubtleCrypto::ExportKey.
            ExportedKey::new_bytes(data.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode privateKeyInfo ASN.1 structure in DER-encoding".into(),
                ))
            })?)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            let mut jwk = JsonWebKey::default();

            // Step 3.2. Set the kty attribute of jwk to "OKP".
            jwk.kty = Some(DOMString::from("OKP"));

            // Step 3.3. Set the crv attribute of jwk to "X448".
            jwk.crv = Some(DOMString::from("X448"));

            // Step 3.4. Set the x attribute of jwk according to the definition in Section 2 of
            // [RFC8037].
            match key.handle() {
                Handle::X448PrivateKey(private_key) => {
                    let public_key = PublicKey::from(private_key);
                    jwk.encode_string_field(JwkStringField::X, public_key.as_bytes());
                },
                Handle::X448PublicKey(public_key) => {
                    jwk.encode_string_field(JwkStringField::X, public_key.as_bytes());
                },
                _ => {
                    return Err(Error::Operation(Some(
                        "[[handle]] internal slot of key is not an X448 key".into(),
                    )));
                },
            }

            // Step 3.5. If the [[type]] internal slot of key is "private"
            //     Set the d attribute of jwk according to the definition in Section 2 of [RFC8037].
            if key.Type() == KeyType::Private {
                if let Handle::X448PrivateKey(private_key) = key.handle() {
                    jwk.encode_string_field(JwkStringField::D, private_key.as_bytes());
                } else {
                    return Err(Error::Operation(None));
                }
                let Handle::X448PrivateKey(private_key) = key.handle() else {
                    return Err(Error::Operation(Some(
                        "[[handle]] internal slot of key is not an X448 private key".into(),
                    )));
                };
                jwk.encode_string_field(JwkStringField::D, private_key.as_bytes().as_slice());
            }

            // Step 3.6. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(&key.usages());

            // Step 3.7. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.8. Let result be the result of converting jwk to an ECMAScript Object, as
            // defined by [WebIDL].
            // NOTE: The conversion to an ECMAScript Object is done by SubtleCrypto::ExportKey.
            ExportedKey::new_jwk(jwk)
        },
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_public => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".into(),
                )));
            }

            // Step 3.2. Let data be an octet string representing the X448 public key represented by
            // the [[handle]] internal slot of key.
            let Handle::X448PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "[[handle]] internal slot of key is not an X448 public key".into(),
                )));
            };
            let data = public_key.as_bytes();

            // Step 3.3. Let result be a new ArrayBuffer associated with the relevant global object
            // of this [HTML], and containing data.
            // NOTE: The conversion to a new ArrayBuffer is done in SubtleCrypto::ExportKey.
            ExportedKey::new_bytes(data.to_vec())
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported export key format for X448".into(),
            )));
        },
    };

    // Step 4. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for X448
pub(crate) fn get_public_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    key: &CryptoKey,
    algorithm: &KeyAlgorithmAndDerivatives,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 9. If usages contains an entry which is not supported for a public key by the algorithm
    // identified by algorithm, then throw a SyntaxError.
    //
    // NOTE: See "impportKey" operation for supported usages
    if !usages.is_empty() {
        return Err(Error::Syntax(Some("Usages is not empty".to_string())));
    }

    // Step 10. Let publicKey be a new CryptoKey representing the public key corresponding to the
    // private key represented by the [[handle]] internal slot of key.
    // Step 11. If an error occurred, then throw a OperationError.
    // Step 12. Set the [[type]] internal slot of publicKey to "public".
    // Step 13. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of publicKey to true.
    // Step 15. Set the [[usages]] internal slot of publicKey to usages.
    let Handle::X448PrivateKey(private_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an X448 private key".into(),
        )));
    };
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        algorithm.clone(),
        usages,
        Handle::X448PublicKey(PublicKey::from(private_key)),
    );

    Ok(public_key)
}
