/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base64ct::{Base64UrlUnpadded, Encoding};
use pkcs8::PrivateKeyInfo;
use pkcs8::der::asn1::{BitStringRef, OctetStringRef};
use pkcs8::der::{AnyRef, Decode};
use pkcs8::spki::{ObjectIdentifier, SubjectPublicKeyInfo};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_X25519, JsonWebKeyExt, KeyAlgorithmAndDerivatives, SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

/// `id-X25519` object identifier defined in [RFC8410]
const X25519_OID_STRING: &str = "1.3.101.110";

const PRIVATE_KEY_LENGTH: usize = 32;
const PUBLIC_KEY_LENGTH: usize = 32;

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
                .map_err(|_| Error::Data)?;

            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the id-X25519 object identifier
            // defined in [RFC8410], then throw a DataError.
            if spki.algorithm.oid != ObjectIdentifier::new_unwrap(X25519_OID_STRING) {
                return Err(Error::Data);
            }

            // Step 2.5. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            if spki.algorithm.parameters.is_some() {
                return Err(Error::Data);
            }

            // Step 2.6. Let publicKey be the X25519 public key identified by the subjectPublicKey
            // field of spki.
            let key_bytes: [u8; PUBLIC_KEY_LENGTH] = spki
                .subject_public_key
                .as_bytes()
                .ok_or(Error::Data)?
                .try_into()
                .map_err(|_| Error::Data)?;
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
            let private_key_info = PrivateKeyInfo::from_der(key_data).map_err(|_| Error::Data)?;

            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the id-X25519 object
            // identifier defined in [RFC8410], then throw a DataError.
            if private_key_info.algorithm.oid != ObjectIdentifier::new_unwrap(X25519_OID_STRING) {
                return Err(Error::Data);
            }

            // Step 2.5. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            if private_key_info.algorithm.parameters.is_some() {
                return Err(Error::Data);
            }

            // Step 2.6. Let curvePrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as the ASN.1 CurvePrivateKey structure specified in Section 7 of [RFC8410], and
            // exactData set to true.
            // Step 2.7. If an error occurred while parsing, then throw a DataError.
            let curve_private_key =
                OctetStringRef::from_der(private_key_info.private_key).map_err(|_| Error::Data)?;
            let key_bytes: [u8; PRIVATE_KEY_LENGTH] = curve_private_key
                .as_bytes()
                .try_into()
                .map_err(|_| Error::Data)?;
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
                return Err(Error::Data);
            }

            // Step 2.2. If the crv field of jwk is not "X25519", then throw a DataError.
            if jwk.crv.as_ref().is_none_or(|crv| crv != ALG_X25519) {
                return Err(Error::Data);
            }

            // Step 2.2. If usages is non-empty and the use field of jwk is present and is not
            // equal to "enc" then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data);
            }

            // Step 2.2. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.2. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data);
            }

            // Step 2.9.
            let (handle, key_type) = match jwk.d {
                Some(d) => {
                    // Step 2.9.1. If jwk does not meet the requirements of the JWK private key
                    // format described in Section 2 of [RFC8037], then throw a DataError.
                    let x = match jwk.x {
                        Some(x) => {
                            Base64UrlUnpadded::decode_vec(&x.str()).map_err(|_| Error::Data)?
                        },
                        None => return Err(Error::Data),
                    };
                    let d = Base64UrlUnpadded::decode_vec(&d.str()).map_err(|_| Error::Data)?;
                    let public_key_bytes: [u8; PUBLIC_KEY_LENGTH] =
                        x.try_into().map_err(|_| Error::Data)?;
                    let private_key_bytes: [u8; PRIVATE_KEY_LENGTH] =
                        d.try_into().map_err(|_| Error::Data)?;
                    let public_key = PublicKey::from(public_key_bytes);
                    let private_key = StaticSecret::from(private_key_bytes);
                    if PublicKey::from(&private_key) != public_key {
                        return Err(Error::Data);
                    }

                    // Step 2.9.1. Let key be a new CryptoKey object that represents the X25519
                    // private key identified by interpreting jwk according to Section 2 of
                    // [RFC8037].
                    // NOTE: CryptoKey is created in Step 2.10 - 2.12.
                    let handle = Handle::X25519PrivateKey(private_key);

                    // Step 2.9.1. Set the [[type]] internal slot of Key to "private".
                    let key_type = KeyType::Private;

                    (handle, key_type)
                },
                // Otherwise:
                None => {
                    // Step 2.9.1. If jwk does not meet the requirements of the JWK public key
                    // format described in Section 2 of [RFC8037], then throw a DataError.
                    let x = match jwk.x {
                        Some(x) => {
                            Base64UrlUnpadded::decode_vec(&x.str()).map_err(|_| Error::Data)?
                        },
                        None => return Err(Error::Data),
                    };
                    let public_key_bytes: [u8; PUBLIC_KEY_LENGTH] =
                        x.try_into().map_err(|_| Error::Data)?;
                    let public_key = PublicKey::from(public_key_bytes);

                    // Step 2.9.1. Let key be a new CryptoKey object that represents the X25519
                    // public key identified by interpreting jwk according to Section 2 of
                    // [RFC8037].
                    // NOTE: CryptoKey is created in Step 2.10 - 2.12.
                    let handle = Handle::X25519PublicKey(public_key);

                    // Step 2.9.1. Set the [[type]] internal slot of Key to "public".
                    let key_type = KeyType::Public;

                    (handle, key_type)
                },
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
        KeyFormat::Raw => {
            // Step 2.1. If usages is not empty then throw a SyntaxError.
            if !usages.is_empty() {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. If the length in bits of keyData is not 256 then throw a DataError.
            if key_data.len() != 32 {
                return Err(Error::Data);
            }

            // Step 2.3. Let algorithm be a new KeyAlgorithm object.
            // Step 2.4. Set the name attribute of algorithm to "X25519".
            // Step 2.5. Let key be a new CryptoKey representing the key data provided in keyData.
            // Step 2.6. Set the [[type]] internal slot of key to "public"
            // Step 2.7. Set the [[algorithm]] internal slot of key to algorithm.
            let key_bytes: [u8; PUBLIC_KEY_LENGTH] =
                key_data.try_into().map_err(|_| Error::Data)?;
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
        // Otherwise: throw a NotSupportedError. (Unreachable)
    };

    // Step 3. Return key
    Ok(key)
}
