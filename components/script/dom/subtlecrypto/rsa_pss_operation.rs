/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rsa::pkcs8::der::Decode;
use rsa::pkcs8::{PrivateKeyInfo, SubjectPublicKeyInfo};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1};
use script_bindings::match_domstring_ascii;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::codegen::UnionTypes::ObjectOrString;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::rsa_common::{self, RsaAlgorithm, decode_base64url_uint};
use crate::dom::subtlecrypto::{
    ALG_RSA_PSS, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, ExportedKey, JsonWebKeyExt,
    KeyAlgorithmAndDerivatives, Operation, SubtleRsaHashedImportParams,
    SubtleRsaHashedKeyAlgorithm, normalize_algorithm,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#rsa-pss-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let (handle, key_type) = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If usages contains an entry which is not "verify", then throw a
            // SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let spki = SubjectPublicKeyInfo::from_der(key_data).map_err(|_| Error::Data)?;

            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the rsaEncryption object
            // identifier defined in [RFC3447], then throw a DataError.
            if spki.algorithm.oid != pkcs1::ALGORITHM_OID {
                return Err(Error::Data);
            }

            // Step 2.5. Let publicKey be the result of performing the parse an ASN.1 structure
            // algorithm, with data as the subjectPublicKeyInfo field of spki, structure as the
            // RSAPublicKey structure specified in Section A.1.1 of [RFC3447], and exactData set to
            // true.
            // Step 2.6. If an error occurred while parsing, or it can be determined that publicKey
            // is not a valid public key according to [RFC3447], then throw a DataError.
            let public_key = RsaPublicKey::try_from(spki).map_err(|_| Error::Data)?;

            // Step 2.7. Let key be a new CryptoKey that represents the RSA public key identified
            // by publicKey.
            // Step 2.8. Set the [[type]] internal slot of key to "public"
            // NOTE: CryptoKey is created in Step 8.
            (Handle::RsaPublicKey(public_key), KeyType::Public)
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If usages contains an entry which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let private_key_info = PrivateKeyInfo::from_der(key_data).map_err(|_| Error::Data)?;

            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the rsaEncryption object
            // identifier defined in [RFC3447], then throw a DataError.
            if private_key_info.algorithm.oid != pkcs1::ALGORITHM_OID {
                return Err(Error::Data);
            }

            // Step 2.5. Let rsaPrivateKey be the result of performing the parse an ASN.1 structure
            // algorithm, with data as the privateKey field of privateKeyInfo, structure as the
            // RSAPrivateKey structure specified in Section A.1.2 of [RFC3447], and exactData set
            // to true.
            // Step 2.6. If an error occurred while parsing, or if rsaPrivateKey is not a valid RSA
            // private key according to [RFC3447], then throw a DataError.
            let rsa_private_key =
                RsaPrivateKey::try_from(private_key_info).map_err(|_| Error::Data)?;

            // Step 2.7. Let key be a new CryptoKey that represents the RSA private key identified
            // by rsaPrivateKey.
            // Step 2.8. Set the [[type]] internal slot of key to "private"
            (Handle::RsaPrivateKey(rsa_private_key), KeyType::Private)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let cx = GlobalScope::get_cx();
            let jwk = JsonWebKey::parse(cx, key_data)?;

            // Step 2.2. If the d field of jwk is present and usages contains an entry which is not
            // "sign", or, if the d field of jwk is not present and usages contains an entry which
            // is not "verify" then throw a SyntaxError.
            if (jwk.d.is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign)) ||
                (jwk.d.is_none() && usages.iter().any(|usage| *usage != KeyUsage::Verify))
            {
                return Err(Error::Syntax(None));
            }

            // Step 2.3. If the kty field of jwk is not a case-sensitive string match to "RSA",
            // then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "RSA") {
                return Err(Error::Data);
            }

            // Step 2.4. If usages is non-empty and the use field of jwk is present and is not a
            // case-sensitive string match to "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data);
            }

            // Step 2.5. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.6. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data);
            }

            // Step 2.7.
            // If the alg field of jwk is not present:
            //     Let hash be undefined.
            // If the alg field is equal to the string "RS1":
            //     Let hash be the string "SHA-1".
            // If the alg field is equal to the string "RS256":
            //     Let hash be the string "SHA-256".
            // If the alg field is equal to the string "RS384":
            //     Let hash be the string "SHA-384".
            // If the alg field is equal to the string "RS512":
            //     Let hash be the string "SHA-512".
            // Otherwise:
            //     Perform any key import steps defined by other applicable specifications, passing
            //     format, jwk and obtaining hash.
            //     If an error occurred or there are no applicable specifications, throw a DataError.
            let hash = match jwk.alg {
                None => None,
                Some(alg) => match_domstring_ascii!(alg,
                    "RS1" => Some(ALG_SHA1),
                    "RS256" => Some(ALG_SHA256),
                    "RS384" => Some(ALG_SHA384),
                    "RS512" => Some(ALG_SHA512),
                    _ => return Err(Error::NotSupported),
                ),
            };

            // Step 2.8. If hash is not undefined:
            if let Some(hash) = hash {
                // Step 2.8.1. Let normalizedHash be the result of normalize an algorithm with alg
                // set to hash and op set to digest.
                let normalized_hash = normalize_algorithm(
                    cx,
                    &Operation::Digest,
                    &ObjectOrString::String(DOMString::from(hash)),
                )?;

                // Step 2.8.2. If normalizedHash is not equal to the hash member of
                // normalizedAlgorithm, throw a DataError.
                if normalized_hash.name() != normalized_algorithm.hash.name {
                    return Err(Error::Data);
                }
            }

            // Step 2.9.
            match jwk.d {
                // If the d field of jwk is present:
                Some(d) => {
                    // Step 2.9.1. If jwk does not meet the requirements of Section 6.3.2 of JSON
                    // Web Algorithms [JWA], then throw a DataError.
                    let n = if let Some(n) = jwk.n {
                        decode_base64url_uint(n)?
                    } else {
                        return Err(Error::Data);
                    };
                    let e = if let Some(e) = jwk.e {
                        decode_base64url_uint(e)?
                    } else {
                        return Err(Error::Data);
                    };
                    let d = decode_base64url_uint(d)?;
                    let mut primes = Vec::new();
                    match (jwk.p, jwk.q, jwk.dp, jwk.dq, jwk.qi) {
                        (Some(p), Some(q), Some(_dp), Some(_dq), Some(_qi)) => {
                            primes.push(decode_base64url_uint(p)?);
                            primes.push(decode_base64url_uint(q)?);
                        },
                        _ => return Err(Error::Data),
                    }
                    if let Some(oth) = jwk.oth {
                        for other_prime in oth {
                            if let (Some(r), Some(_d), Some(_t)) =
                                (other_prime.r, other_prime.d, other_prime.t)
                            {
                                primes.push(decode_base64url_uint(r)?)
                            } else {
                                return Err(Error::Data);
                            }
                        }
                    }

                    // Step 2.9.2. Let privateKey represents the RSA private key identified by
                    // interpreting jwk according to Section 6.3.2 of JSON Web Algorithms [JWA].
                    // Step 2.9.3. If privateKey is not a valid RSA private key according to
                    // [RFC3447], then throw a DataError.
                    let private_key =
                        RsaPrivateKey::from_components(n, e, d, primes).map_err(|_| Error::Data)?;

                    // Step 2.9.4. Let key be a new CryptoKey object that represents privateKey.
                    // Step 2.9.5. Set the [[type]] internal slot of key to "private"
                    // NOTE: CryptoKey is created in Step 8.
                    (Handle::RsaPrivateKey(private_key), KeyType::Private)
                },
                // Otherwise:
                None => {
                    // Step 2.9.1. If jwk does not meet the requirements of Section 6.3.1 of JSON
                    // Web Algorithms [JWA], then throw a DataError.
                    let n = if let Some(n) = jwk.n {
                        decode_base64url_uint(n)?
                    } else {
                        return Err(Error::Data);
                    };
                    let e = if let Some(e) = jwk.e {
                        decode_base64url_uint(e)?
                    } else {
                        return Err(Error::Data);
                    };

                    // Step 2.9.2. Let publicKey represent the RSA public key identified by
                    // interpreting jwk according to Section 6.3.1 of JSON Web Algorithms [JWA].
                    // Step 2.9.3. If publicKey can be determined to not be a valid RSA public key
                    // according to [RFC3447], then throw a DataError.
                    let public_key = RsaPublicKey::new(n, e).map_err(|_| Error::Data)?;

                    // Step 2.9.4. Let key be a new CryptoKey representing publicKey.
                    // Step 2.9.5. Set the [[type]] internal slot of key to "public"
                    // NOTE: CryptoKey is created in Step 8.
                    (Handle::RsaPublicKey(public_key), KeyType::Public)
                },
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported);
        },
    };

    // Step 3. Let algorithm be a new RsaHashedKeyAlgorithm dictionary.
    // Step 4. Set the name attribute of algorithm to "RSA-PSS"
    // Step 5. Set the modulusLength attribute of algorithm to the length, in bits, of the RSA
    // public modulus.
    // Step 6. Set the publicExponent attribute of algorithm to the BigInteger representation of
    // the RSA public exponent.
    // Step 7. Set the hash attribute of algorithm to the hash member of normalizedAlgorithm.
    let (modulus_length, public_exponent) = match &handle {
        Handle::RsaPrivateKey(rsa_private_key) => (
            rsa_private_key.size() as u32 * 8,
            rsa_private_key.e().to_bytes_be(),
        ),
        Handle::RsaPublicKey(rsa_public_key) => (
            rsa_public_key.size() as u32 * 8,
            rsa_public_key.e().to_bytes_be(),
        ),
        _ => unreachable!(),
    };
    let algorithm = SubtleRsaHashedKeyAlgorithm {
        name: ALG_RSA_PSS.to_string(),
        modulus_length,
        public_exponent,
        hash: normalized_algorithm.hash.clone(),
    };

    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let key = CryptoKey::new(
        global,
        key_type,
        extractable,
        KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm),
        usages,
        handle,
        can_gc,
    );

    // Step 9. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#rsa-pss-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    rsa_common::export_key(RsaAlgorithm::RsaPss, format, key)
}
