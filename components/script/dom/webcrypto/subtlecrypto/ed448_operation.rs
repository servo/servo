/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ed448_goldilocks::elliptic_curve::Generate;
use ed448_goldilocks::elliptic_curve::group::cofactor::CofactorGroup;
use ed448_goldilocks::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePublicKey};
use ed448_goldilocks::signature::SignatureEncoding;
use ed448_goldilocks::{CompressedEdwardsY, PublicKeyBytes, Signature, SigningKey, VerifyingKey};
use js::context::JSContext;
use pkcs8::der::Encode;
use pkcs8::der::asn1::OctetStringRef;
use pkcs8::{AlgorithmIdentifierRef, ObjectIdentifier, PrivateKeyInfoRef};
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
    SubtleEd448Params, SubtleKeyAlgorithm,
};

/// `id-Ed448` object identifier defined in [RFC8410]
const ED448_OID_STRING: &str = "1.3.101.113";

/// <https://wicg.github.io/webcrypto-secure-curves/#ed448-operations>
pub(crate) fn sign(
    normalized_algorithm: &SubtleEd448Params,
    key: &CryptoKey,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"private\"".into(),
        )));
    }

    // Step 2. Let context be the contents of the context member of normalizedAlgorithm or the empty
    // octet string if the context member of normalizedAlgorithm is not present.
    let context = normalized_algorithm.context.as_deref().unwrap_or_default();

    // Step 3. If context has a length greater than 255 bytes, then throw an OperationError.
    if context.len() > 255 {
        return Err(Error::Operation(Some(
            "Context has a length greater than 255 bytes".into(),
        )));
    }

    // Step 4. Perform the Ed448 signing process, as specified in [RFC8032], Section 5.2.6, with
    // message as M and context as C, using the Ed448 private key associated with key.
    let Handle::Ed448PrivateKey(private_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an Ed448 private key".into(),
        )));
    };
    let result = private_key.sign_ctx(context, message).map_err(|_| {
        Error::Operation(Some(
            "Failed to sign the message with Ed448 algorithm".into(),
        ))
    })?;

    // Step 5. Return a new ArrayBuffer associated with the relevant global object of this [HTML],
    // and containing the bytes of the signature resulting from performing the Ed448 signing
    // process.
    // NOTE: The conversion to ArrayBuffer is done in SubtleCrypto::Sign.
    Ok(result.to_vec())
}

/// <https://wicg.github.io/webcrypto-secure-curves/#ed448-operations>
pub(crate) fn verify(
    normalized_algorithm: &SubtleEd448Params,
    key: &CryptoKey,
    message: &[u8],
    signature: &[u8],
) -> Result<bool, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".into(),
        )));
    }

    // Step 2. Let context be the contents of the context member of normalizedAlgorithm or the empty
    // octet string if the context member of normalizedAlgorithm is not present.
    let context = normalized_algorithm.context.as_deref().unwrap_or_default();

    // Step 3. If context has a length greater than 255 bytes, then throw an OperationError.
    if context.len() > 255 {
        return Err(Error::Operation(Some(
            "Context has a length greater than 255 bytes".into(),
        )));
    }

    // Step 4. If the key data of key represents an invalid point or a small-order element on the
    // Elliptic Curve of Ed448, return false.
    let Handle::Ed448PublicKey(public_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an Ed448 public key".into(),
        )));
    };
    if (*public_key).to_edwards().is_small_order().into() {
        return Ok(false);
    }

    // Step 5. If the point R, encoded in the first half of signature, represents an invalid point
    // or a small-order element on the Elliptic Curve of Ed448, return false.
    if CompressedEdwardsY::try_from(&signature[..signature.len() / 2])
        .ok()
        .and_then(|compressed_point| compressed_point.decompress().into_option())
        .map(|point| point.to_edwards().is_small_order().into())
        .unwrap_or(true)
    {
        return Ok(false);
    }

    // Step 6. Perform the Ed448 verification steps, as specified in [RFC8032], Section 5.2.7, using
    // the cofactorless (unbatched) equation, [S]B = R + [k]A', on the signature, with message as M
    // and context as C, using the Ed448 public key associated with key.
    // Step 7. Let result be a boolean with the value true if the signature is valid and the value
    // false otherwise.
    let result = Signature::from_slice(signature)
        .and_then(|signature| public_key.verify_ctx(&signature, context, message))
        .is_ok();

    // Step 8. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-secure-curves/#ed448-operations>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains a value which is not one of "sign" or "verify", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"sign\" or \"verify\"".into(),
        )));
    }

    // Step 2. Generate an Ed448 key pair, as defined in [RFC8032], section 5.1.5.
    let private_key = SigningKey::try_generate()
        .map_err(|_| Error::Operation(Some("Failed to generate Ed448 private key".into())))?;
    let public_key = private_key.verifying_key();

    // Step 3. Let algorithm be a new KeyAlgorithm object.
    // Step 4. Set the name attribute of algorithm to "Ed448".
    let algorithm = SubtleKeyAlgorithm {
        name: CryptoAlgorithm::Ed448,
    };

    // Step 5. Let publicKey be a new CryptoKey associated with the relevant global object of this
    // [HTML], and representing the public key of the generated key pair.
    // Step 6. Set the [[type]] internal slot of publicKey to "public"
    // Step 7. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 8. Set the [[extractable]] internal slot of publicKey to true.
    // Step 9. Set the [[usages]] internal slot of publicKey to be the usage intersection of usages
    // and [ "verify" ].
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
        usages
            .iter()
            .filter(|&usage| *usage == KeyUsage::Verify)
            .cloned()
            .collect(),
        Handle::Ed448PublicKey(public_key),
    );

    // Step 10. Let privateKey be a new CryptoKey associated with the relevant global object of this
    // [HTML], and representing the private key of the generated key pair.
    // Step 11. Set the [[type]] internal slot of privateKey to "private"
    // Step 12. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 13. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 14. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "sign" ].
    let private_key = CryptoKey::new(
        cx,
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages
            .iter()
            .filter(|&usage| *usage == KeyUsage::Sign)
            .cloned()
            .collect(),
        Handle::Ed448PrivateKey(private_key),
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

/// <https://wicg.github.io/webcrypto-secure-curves/#ed448-operations>
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
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"verify\"".into(),
                )));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the id-Ed448 object identifier
            // defined in [RFC8410], then throw a DataError.
            // Step 2.5. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            // Step 2.6. Let publicKey be the Ed448 public key identified by the subjectPublicKey
            // field of spki.
            let public_key = VerifyingKey::from_public_key_der(key_data).map_err(|_| {
                Error::Data(Some(
                    "Failed to parse the Ed448 public key in SPKI format".into(),
                ))
            })?;

            // Step 2.7. Let key be a new CryptoKey associated with the relevant global object of
            // this [HTML], and that represents publicKey.
            // Step 2.8. Set the [[type]] internal slot of key to "public"
            // Step 2.9. Let algorithm be a new KeyAlgorithm.
            // Step 2.10. Set the name attribute of algorithm to "Ed448".
            // Step 2.11. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::Ed448,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::Ed448PublicKey(public_key),
            )
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If usages contains a value which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\"".into(),
                )));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the id-Ed448 object
            // identifier defined in [RFC8410], then throw a DataError.
            // Step 2.5. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            // Step 2.6. Let curvePrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as the ASN.1 CurvePrivateKey structure specified in Section 7 of [RFC8410], and
            // exactData set to true.
            // Step 2.7. If an error occurred while parsing, then throw a DataError.
            let curve_private_key = SigningKey::from_pkcs8_der(key_data).map_err(|_| {
                Error::Data(Some(
                    "Failed to parse the Ed448 private key in PKCS#8 format".into(),
                ))
            })?;

            // Step 2.8. Let key be a new CryptoKey associated with the relevant global object of
            // this [HTML], and that represents the Ed448 private key identified by curvePrivateKey.
            // Step 2.9. Set the [[type]] internal slot of key to "private"
            // Step 2.10. Let algorithm be a new KeyAlgorithm.
            // Step 2.11. Set the name attribute of algorithm to "Ed448".
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::Ed448,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::Ed448PrivateKey(curve_private_key),
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

            // Step 2.2. If the d field is present and usages contains a value which is not "sign",
            // or, if the d field is not present and usages contains a value which is not "verify"
            // then throw a SyntaxError.
            if jwk.d.is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "The d field is present and if usages contains an entry which is not \
                        \"sign\""
                        .into(),
                )));
            }
            if jwk.d.is_none() && usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "The d field is not present and if usages contains a value which is not \
                        \"verify\""
                        .into(),
                )));
            }

            // Step 2.3. If the kty field of jwk is not "OKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "OKP") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"OKP\"".into(),
                )));
            }

            // Step 2.4. If the crv field of jwk is not "Ed448", then throw a DataError.
            if jwk.crv.as_ref().is_none_or(|crv| crv != "Ed448") {
                return Err(Error::Data(Some(
                    "The crv field of jwk is not \"Ed448\"".into(),
                )));
            }

            // Step 2.5. If the alg field of jwk is present and is not "Ed448" or "EdDSA", then
            // throw a DataError.
            if jwk
                .alg
                .as_ref()
                .is_some_and(|alg| !matches!(alg.str().as_ref(), "Ed448" | "EdDSA"))
            {
                return Err(Error::Data(Some(
                    "The 'alg' field is different from 'Ed448' and 'EdDSA'".into(),
                )));
            }

            // Step 2.6. If usages is non-empty and the use field of jwk is present and is not
            // "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not equal to \
                        \"sig\""
                        .into(),
                )));
            }

            // Step 2.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.as_ref().is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some(
                    "The ext field of jwk is present and has the value false \
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
                let private_key_bytes = d.as_slice();
                let public_key_bytes = PublicKeyBytes(x.as_slice().try_into().map_err(|_| {
                    Error::Data(Some("Invalid length of public key in 'x' field".into()))
                })?);
                let private_key = SigningKey::try_from(private_key_bytes).map_err(|_| {
                    Error::Data(Some("Failed to import private key from 'd' field".into()))
                })?;
                let public_key = VerifyingKey::try_from(public_key_bytes).map_err(|_| {
                    Error::Data(Some("Failed to import public key from 'x' field".into()))
                })?;
                if private_key.verifying_key() != public_key {
                    return Err(Error::Data(Some(
                        "Public key in 'x' field does not match private key in 'd' field".into(),
                    )));
                };

                // Step 2.9.2. Let key be a new CryptoKey object that represents the Ed448 private
                // key identified by interpreting jwk according to Section 2 of [RFC8037].
                // NOTE: The CryptoKey is created in Step 2.10 - Step 2.12.
                let handle = Handle::Ed448PrivateKey(private_key);

                // Step 2.9.3. Set the [[type]] internal slot of Key to "private".
                let key_type = KeyType::Private;

                (handle, key_type)
            }
            // Otherwise:
            else {
                // Step 2.9.1. If jwk does not meet the requirements of the JWK public key format
                // described in Section 2 of [RFC8037], then throw a DataError.
                let x = jwk.decode_required_string_field(JwkStringField::X)?;
                let public_key_bytes = PublicKeyBytes(x.as_slice().try_into().map_err(|_| {
                    Error::Data(Some("Invalid length of public key in 'x' field".into()))
                })?);
                let public_key = VerifyingKey::try_from(public_key_bytes).map_err(|_| {
                    Error::Data(Some("Failed to import public key from 'x' field".into()))
                })?;

                // Step 2.9.2. Let key be a new CryptoKey object that represents the Ed448 public
                // key identified by interpreting jwk according to Section 2 of [RFC8037].
                // NOTE: The CryptoKey is created in Step 2.10 - Step 2.12.
                let handle = Handle::Ed448PublicKey(public_key);

                // Step 2.9.3. Set the [[type]] internal slot of Key to "public".
                let key_type = KeyType::Public;

                (handle, key_type)
            };

            // Step 2.10. Let algorithm be a new instance of a KeyAlgorithm object.
            // Step 2.11. Set the name attribute of algorithm to "Ed448".
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::Ed448,
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
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not one of \"verify\"".into(),
                )));
            }

            // Step 2.2. Let data be keyData.
            let data = key_data;

            // Step 2.3. If the length in bits of data is not 448 then throw a DataError.
            // NOTE: It should be "not 456", instead of "not 448", according to
            // <https://www.rfc-editor.org/info/rfc8032/#section-5.2.5>
            if data.len() != 57 {
                return Err(Error::Data(Some("The key length is not 456 bits".into())));
            }

            // Step 2.4. Let algorithm be a new KeyAlgorithm object.
            // Step 2.5. Set the name attribute of algorithm to "Ed448".
            let algorithm = SubtleKeyAlgorithm {
                name: CryptoAlgorithm::Ed448,
            };

            // Step 2.6. Let key be a new CryptoKey associated with the relevant global object of
            // this [HTML], and that represents data.
            // Step 2.7. Set the [[type]] internal slot of key to "public"
            // Step 2.8. Set the [[algorithm]] internal slot of key to algorithm.
            let public_key_bytes =
                PublicKeyBytes(data.try_into().map_err(|_| {
                    Error::Data(Some("Invalid length of public key raw bytes".into()))
                })?);
            let public_key = VerifyingKey::try_from(public_key_bytes).map_err(|_| {
                Error::Data(Some("Failed to import public key from raw bytes".into()))
            })?;
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::Ed448PublicKey(public_key),
            )
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported import key format for Ed448".into(),
            )));
        },
    };

    // Step 3. Return key
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-secure-curves/#ed448-operations>
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
            //     * Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //       properties:
            //         * Set the algorithm object identifier to the id-Ed448 OID defined in
            //         [RFC8410].
            //     * Set the subjectPublicKey field to keyData.
            let Handle::Ed448PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "[[handle]] internal slot of key is not an Ed448 public key".into(),
                )));
            };
            let data = public_key.to_public_key_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to convert Ed448 public key to subjectPublicKeyInfo ASN.1 structure"
                        .into(),
                ))
            })?;

            // Step 3.3. Let result be a new ArrayBuffer associated with the relevant global object
            // of this [HTML], and containing data.
            // NOTE: The conversion to a new ArrayBuffer is done in SubtleCrypto::ExportKey.
            ExportedKey::new_bytes(data.into_vec())
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
            //     * Set the version field to 0.
            //     * Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //       with the following properties:
            //         * Set the algorithm object identifier to the id-Ed448 OID defined in
            //         [RFC8410].
            //     * Set the privateKey field to the result of DER-encoding a CurvePrivateKey ASN.1
            //       type, as defined in Section 7 of [RFC8410], that represents the Ed448 private
            //       key represented by the [[handle]] internal slot of key
            //
            // NOTE: If we directly call `EncodePrivateKey::to_pkcs8_der` on `private_key`, the
            // resultant PKCS#8 document will include the public key, which does not match the
            // specification. Therefore, we manually construct the PrivateKeyInfoRef.
            let Handle::Ed448PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "[[handle]] internal slot of key is not an Ed448 private key".into(),
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
                    oid: ObjectIdentifier::new_unwrap(ED448_OID_STRING),
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

            // Step 3.3. Set the alg attribute of jwk to "Ed448".
            jwk.alg = Some(DOMString::from("Ed448"));

            // Step 3.4. Set the crv attribute of jwk to "Ed448".
            jwk.crv = Some(DOMString::from("Ed448"));

            // Step 3.5. Set the x attribute of jwk according to the definition in Section 2 of
            // [RFC8037].
            match key.handle() {
                Handle::Ed448PrivateKey(private_key) => {
                    jwk.encode_string_field(
                        JwkStringField::X,
                        private_key.verifying_key().as_bytes().as_slice(),
                    );
                },
                Handle::Ed448PublicKey(public_key) => {
                    jwk.encode_string_field(JwkStringField::X, public_key.as_bytes().as_slice());
                },
                _ => {
                    return Err(Error::Operation(Some(
                        "[[handle]] internal slot of key is not an Ed448 key".into(),
                    )));
                },
            }

            // Step 3.6. If the [[type]] internal slot of key is "private"
            //     Set the d attribute of jwk according to the definition in Section 2 of [RFC8037].
            if key.Type() == KeyType::Private {
                let Handle::Ed448PrivateKey(private_key) = key.handle() else {
                    return Err(Error::Operation(Some(
                        "[[handle]] internal slot of key is not an Ed448 private key".into(),
                    )));
                };
                jwk.encode_string_field(JwkStringField::D, private_key.as_bytes().as_slice());
            }

            // Step 3.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(&key.usages());

            // Step 3.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.9. Let result be the result of converting jwk to an ECMAScript Object, as
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

            // Step 3.2. Let data be an octet string representing the Ed448 public key represented
            // by the [[handle]] internal slot of key.
            let Handle::Ed448PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "[[handle]] internal slot of key is not an Ed448 public key".into(),
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
                "Unsupported export key format for Ed448".into(),
            )));
        },
    };

    // Step 4. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for Ed448
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
    if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"verify\"".into(),
        )));
    }

    // Step 10. Let publicKey be a new CryptoKey representing the public key corresponding to the
    // private key represented by the [[handle]] internal slot of key.
    // Step 11. If an error occurred, then throw a OperationError.
    // Step 12. Set the [[type]] internal slot of publicKey to "public".
    // Step 13. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of publicKey to true.
    // Step 15. Set the [[usages]] internal slot of publicKey to usages.
    let Handle::Ed448PrivateKey(private_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an Ed448 private key".into(),
        )));
    };
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        algorithm.clone(),
        usages,
        Handle::Ed448PublicKey(private_key.verifying_key()),
    );

    Ok(public_key)
}
