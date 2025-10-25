/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base64::Engine;
use rsa::BigUint;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use rsa::traits::{PrivateKeyParts, PublicKeyParts};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{CryptoKeyMethods, KeyType};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    JsonWebKey, KeyFormat, RsaOtherPrimesInfo,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::subtlecrypto::{ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, ExportedKey};

pub(crate) enum RsaAlgorithm {
    RsaSsaPkcs1v15,
    RsaPss,
    RsaOaep,
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-export-key>
/// <https://w3c.github.io/webcrypto/#rsa-pss-operations-export-key>
/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-export-key>
///
/// The steps that are different among RSA algorithms are differentiated by match arms on
/// `rsa_algorithm` parameters.
pub(crate) fn export_key(
    rsa_algorithm: RsaAlgorithm,
    format: KeyFormat,
    key: &CryptoKey,
) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the key to be exported.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    // Step 3.
    let result = match format {
        // If format is "spki"
        KeyFormat::Spki => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess);
            }

            // Step 3.2.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //     Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //     properties:
            //         Set the algorithm field to the OID rsaEncryption defined in [RFC3447].
            //         Set the params field to the ASN.1 type NULL.
            //     Set the subjectPublicKey field to the result of DER-encoding an RSAPublicKey
            //     ASN.1 type, as defined in [RFC3447], Appendix A.1.1, that represents the RSA
            //     public key represented by the [[handle]] internal slot of key
            let Handle::RsaPublicKey(public_key) = key.handle() else {
                return Err(Error::Operation);
            };
            let data = public_key
                .to_public_key_der()
                .map_err(|_| Error::Operation)?;

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Raw(data.to_vec())
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess);
            }

            // Step 3.2.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //     Set the version field to 0.
            //     Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //     with the following properties:
            //         Set the algorithm field to the OID rsaEncryption defined in [RFC3447].
            //         Set the params field to the ASN.1 type NULL.
            //     Set the privateKey field to the result of DER-encoding an RSAPrivateKey ASN.1
            //     type, as defined in [RFC3447], Appendix A.1.2, that represents the RSA private
            //     key represented by the [[handle]] internal slot of key
            let Handle::RsaPrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation);
            };
            let data = private_key.to_pkcs8_der().map_err(|_| Error::Operation)?;

            // Step 3.3.  Let result be the result of DER-encoding data.
            ExportedKey::Raw(data.as_bytes().to_vec())
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            // Step 3.2. Set the kty attribute of jwk to the string "RSA".
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("RSA")),
                ..Default::default()
            };

            // Step 3.3. Let hash be the name attribute of the hash attribute of the [[algorithm]]
            // internal slot of key.
            let hash = key.algorithm().hash()?.name.as_str();

            match &rsa_algorithm {
                RsaAlgorithm::RsaSsaPkcs1v15 => {
                    // Step 3.4.
                    // If hash is "SHA-1":
                    //     Set the alg attribute of jwk to the string "RS1".
                    // If hash is "SHA-256":
                    //     Set the alg attribute of jwk to the string "RS256".
                    // If hash is "SHA-384":
                    //     Set the alg attribute of jwk to the string "RS384".
                    // If hash is "SHA-512":
                    //     Set the alg attribute of jwk to the string "RS512".
                    // Otherwise:
                    //     Perform any key export steps defined by other applicable specifications,
                    //     passing format, key and obtaining alg.
                    //     Set the alg attribute of jwk to alg.
                    jwk.alg = match hash {
                        ALG_SHA1 => Some(DOMString::from("RS1")),
                        ALG_SHA256 => Some(DOMString::from("RS256")),
                        ALG_SHA384 => Some(DOMString::from("RS384")),
                        ALG_SHA512 => Some(DOMString::from("RS512")),
                        _ => return Err(Error::NotSupported),
                    };
                },
                RsaAlgorithm::RsaPss => {
                    // Step 3.4.
                    // If hash is "SHA-1":
                    //     Set the alg attribute of jwk to the string "RS1".
                    // If hash is "SHA-256":
                    //     Set the alg attribute of jwk to the string "RS256".
                    // If hash is "SHA-384":
                    //     Set the alg attribute of jwk to the string "RS384".
                    // If hash is "SHA-512":
                    //     Set the alg attribute of jwk to the string "RS512".
                    // Otherwise:
                    //     Perform any key export steps defined by other applicable specifications,
                    //     passing format and the hash attribute of the [[algorithm]] internal slot
                    //     of key and obtaining alg.
                    //     If an error occurred or there are no applicable specifications, throw a
                    //     NotSupportedError.
                    //     Set the alg attribute of jwk to alg.
                    jwk.alg = match hash {
                        ALG_SHA1 => Some(DOMString::from("RS1")),
                        ALG_SHA256 => Some(DOMString::from("RS256")),
                        ALG_SHA384 => Some(DOMString::from("RS384")),
                        ALG_SHA512 => Some(DOMString::from("RS512")),
                        _ => return Err(Error::NotSupported),
                    };
                },
                RsaAlgorithm::RsaOaep => {
                    // Step 3.4.
                    // If hash is "SHA-1":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP".
                    // If hash is "SHA-256":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP-256".
                    // If hash is "SHA-384":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP-384".
                    // If hash is "SHA-512":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP-512".
                    // Otherwise:
                    //     Perform any key export steps defined by other applicable specifications,
                    //     passing format and the hash attribute of the [[algorithm]] internal slot
                    //     of key and obtaining alg.
                    //     Set the alg attribute of jwk to alg.
                    jwk.alg = match hash {
                        ALG_SHA1 => Some(DOMString::from("RSA-OAEP")),
                        ALG_SHA256 => Some(DOMString::from("RSA-OAEP-256")),
                        ALG_SHA384 => Some(DOMString::from("RSA-OAEP-384")),
                        ALG_SHA512 => Some(DOMString::from("RSA-OAEP-512")),
                        _ => return Err(Error::NotSupported),
                    };
                },
            }

            // Step 3.5. Set the attributes n and e of jwk according to the corresponding
            // definitions in JSON Web Algorithms [JWA], Section 6.3.1.
            match key.handle() {
                Handle::RsaPrivateKey(private_key) => {
                    jwk.n = Some(encode_base64url_uint(private_key.n()));
                    jwk.e = Some(encode_base64url_uint(private_key.e()));
                },
                Handle::RsaPublicKey(public_key) => {
                    jwk.n = Some(encode_base64url_uint(public_key.n()));
                    jwk.e = Some(encode_base64url_uint(public_key.e()));
                },
                _ => return Err(Error::Operation),
            };

            // Step 3.6. If the [[type]] internal slot of key is "private":
            if key.Type() == KeyType::Private {
                // Step 3.6.1. Set the attributes named d, p, q, dp, dq, and qi of jwk according to
                // the corresponding definitions in JSON Web Algorithms [JWA], Section 6.3.2.
                let Handle::RsaPrivateKey(private_key) = key.handle() else {
                    return Err(Error::Operation);
                };
                jwk.d = Some(encode_base64url_uint(private_key.d()));

                let primes = private_key.primes();
                if let (Some(p), Some(q), Some(dp), Some(dq), Some(qi)) = (
                    primes.first(),
                    primes.get(1),
                    private_key.dp(),
                    private_key.dq(),
                    &private_key.crt_coefficient(),
                ) {
                    jwk.p = Some(encode_base64url_uint(p));
                    jwk.q = Some(encode_base64url_uint(q));
                    jwk.dp = Some(encode_base64url_uint(dp));
                    jwk.dq = Some(encode_base64url_uint(dq));
                    jwk.qi = Some(encode_base64url_uint(qi));
                }

                // Step 3.6.2. If the underlying RSA private key represented by the [[handle]]
                // internal slot of key is represented by more than two primes, set the attribute
                // named oth of jwk according to the corresponding definition in JSON Web
                // Algorithms [JWA], Section 6.3.2.7
                // FIXME: Calculate d and t. Fix this when rsa 0.10 is released with a new library
                // to handle big integer.
                let mut oth = Vec::new();
                for prime in primes.iter().skip(2) {
                    oth.push(RsaOtherPrimesInfo {
                        r: Some(encode_base64url_uint(prime)),
                        d: None,
                        t: None,
                    });
                }
                if !oth.is_empty() {
                    jwk.oth = Some(oth);
                }
            }

            // Step 3.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.key_ops = Some(
                key.usages()
                    .iter()
                    .map(|usage| DOMString::from(usage.as_str()))
                    .collect::<Vec<DOMString>>(),
            );

            // Step 3.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.9. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // Otherwise
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported);
        },
    };

    // Step 4. Return result.
    Ok(result)
}

/// Encode a BigUint to a Base64urlUInt-encoded representation.
pub(crate) fn encode_base64url_uint(integer: &BigUint) -> DOMString {
    let bytes = integer.to_bytes_be();
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(&bytes)
        .into()
}

/// Decode a Base64urlUInt-encoded positive or zero integer value from a DOMString. If it fails to
/// decode the input, throw a DataError.
pub(crate) fn decode_base64url_uint(input: DOMString) -> Result<BigUint, Error> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&*input.str())
        .map_err(|_| Error::Data)?;
    Ok(BigUint::from_bytes_be(&bytes))
}
