/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webcrypto/#subtlecrypto-interface

// enum KeyFormat { "raw", "spki", "pkcs8", "jwk" };

[SecureContext,Exposed=(Window,Worker),Pref="dom_crypto_subtle_enabled"]
interface SubtleCrypto {
  Promise<any> encrypt(AlgorithmIdentifier algorithm,
                       CryptoKey key,
                       BufferSource data);
  Promise<any> decrypt(AlgorithmIdentifier algorithm,
                       CryptoKey key,
                       BufferSource data);
  Promise<any> sign(AlgorithmIdentifier algorithm,
                    CryptoKey key,
                    BufferSource data);
  Promise<any> verify(AlgorithmIdentifier algorithm,
                      CryptoKey key,
                      BufferSource signature,
                      BufferSource data);
  Promise<any> digest(AlgorithmIdentifier algorithm,
                      BufferSource data);

  Promise<any> generateKey(AlgorithmIdentifier algorithm,
                          boolean extractable,
                          sequence<KeyUsage> keyUsages );
  Promise<any> deriveKey(AlgorithmIdentifier algorithm,
                         CryptoKey baseKey,
                         AlgorithmIdentifier derivedKeyType,
                         boolean extractable,
                         sequence<KeyUsage> keyUsages );
  Promise<ArrayBuffer> deriveBits(AlgorithmIdentifier algorithm,
                          CryptoKey baseKey,
                          optional unsigned long? length = null);

  Promise<CryptoKey> importKey(KeyFormat format,
                         (BufferSource or JsonWebKey) keyData,
                         AlgorithmIdentifier algorithm,
                         boolean extractable,
                         sequence<KeyUsage> keyUsages );
  Promise<any> exportKey(KeyFormat format, CryptoKey key);

  Promise<any> wrapKey(KeyFormat format,
                       CryptoKey key,
                       CryptoKey wrappingKey,
                       AlgorithmIdentifier wrapAlgorithm);
  Promise<CryptoKey> unwrapKey(KeyFormat format,
                         BufferSource wrappedKey,
                         CryptoKey unwrappingKey,
                         AlgorithmIdentifier unwrapAlgorithm,
                         AlgorithmIdentifier unwrappedKeyAlgorithm,
                         boolean extractable,
                         sequence<KeyUsage> keyUsages );
};

// https://w3c.github.io/webcrypto/#big-integer

typedef Uint8Array BigInteger;

// https://w3c.github.io/webcrypto/#algorithm-dictionary

typedef (object or DOMString) AlgorithmIdentifier;

typedef AlgorithmIdentifier HashAlgorithmIdentifier;

dictionary Algorithm {
  required DOMString name;
};

// https://w3c.github.io/webcrypto/#key-algorithm-dictionary

dictionary KeyAlgorithm {
  required DOMString name;
};

// https://w3c.github.io/webcrypto/#RsaKeyGenParams-dictionary

dictionary RsaKeyGenParams : Algorithm {
  required [EnforceRange] unsigned long modulusLength;
  required BigInteger publicExponent;
};

// https://w3c.github.io/webcrypto/#RsaHashedKeyGenParams-dictionary

dictionary RsaHashedKeyGenParams : RsaKeyGenParams {
  required HashAlgorithmIdentifier hash;
};

// https://w3c.github.io/webcrypto/#RsaKeyAlgorithm-dictionary

dictionary RsaKeyAlgorithm : KeyAlgorithm {
  required unsigned long modulusLength;
  required BigInteger publicExponent;
};

// https://w3c.github.io/webcrypto/#RsaHashedKeyAlgorithm-dictionary

dictionary RsaHashedKeyAlgorithm : RsaKeyAlgorithm {
  required KeyAlgorithm hash;
};

// https://w3c.github.io/webcrypto/#RsaHashedImportParams-dictionary

dictionary RsaHashedImportParams : Algorithm {
  required HashAlgorithmIdentifier hash;
};

// https://w3c.github.io/webcrypto/#EcdsaParams-dictionary
dictionary EcdsaParams : Algorithm {
  required HashAlgorithmIdentifier hash;
};

// https://w3c.github.io/webcrypto/#EcKeyGenParams-dictionary

typedef DOMString NamedCurve;

dictionary EcKeyGenParams : Algorithm {
  required NamedCurve namedCurve;
};

// https://w3c.github.io/webcrypto/#EcKeyAlgorithm-dictionary

dictionary EcKeyAlgorithm : KeyAlgorithm {
  required NamedCurve namedCurve;
};

// https://w3c.github.io/webcrypto/#EcKeyImportParams-dictionary

dictionary EcKeyImportParams : Algorithm {
  required NamedCurve namedCurve;
};

// https://w3c.github.io/webcrypto/#dh-EcdhKeyDeriveParams

dictionary EcdhKeyDeriveParams : Algorithm {
  required CryptoKey public;
};

// https://w3c.github.io/webcrypto/#aes-ctr-params

dictionary AesCtrParams : Algorithm {
  required BufferSource counter;
  required [EnforceRange] octet length;
};

// https://w3c.github.io/webcrypto/#AesKeyAlgorithm-dictionary

dictionary AesKeyAlgorithm : KeyAlgorithm {
  required unsigned short length;
};

// https://w3c.github.io/webcrypto/#aes-keygen-params

dictionary AesKeyGenParams : Algorithm {
  required [EnforceRange] unsigned short length;
};

// https://w3c.github.io/webcrypto/#aes-derivedkey-params

dictionary AesDerivedKeyParams : Algorithm {
  required [EnforceRange] unsigned short length;
};

// https://w3c.github.io/webcrypto/#aes-cbc-params

dictionary AesCbcParams : Algorithm {
  required BufferSource iv;
};

// https://w3c.github.io/webcrypto/#aes-gcm-params

dictionary AesGcmParams : Algorithm {
  required BufferSource iv;
  BufferSource additionalData;
  [EnforceRange] octet tagLength;
};

// https://w3c.github.io/webcrypto/#hmac-importparams

dictionary HmacImportParams : Algorithm {
  required HashAlgorithmIdentifier hash;
  [EnforceRange] unsigned long length;
};

// https://w3c.github.io/webcrypto/#HmacKeyAlgorithm-dictionary

dictionary HmacKeyAlgorithm : KeyAlgorithm {
  required KeyAlgorithm hash;
  required unsigned long length;
};

// https://w3c.github.io/webcrypto/#hmac-keygen-params

dictionary HmacKeyGenParams : Algorithm {
  required HashAlgorithmIdentifier hash;
  [EnforceRange] unsigned long length;
};

// https://w3c.github.io/webcrypto/#hkdf-params

dictionary HkdfParams : Algorithm {
  required HashAlgorithmIdentifier hash;
  required BufferSource salt;
  required BufferSource info;
};

// https://w3c.github.io/webcrypto/#pbkdf2-params

dictionary Pbkdf2Params : Algorithm {
  required BufferSource salt;
  required [EnforceRange] unsigned long iterations;
  required HashAlgorithmIdentifier hash;
};

// https://w3c.github.io/webcrypto/#JsonWebKey-dictionary

dictionary RsaOtherPrimesInfo {
  // The following fields are defined in Section 6.3.2.7 of JSON Web Algorithms
  DOMString r;
  DOMString d;
  DOMString t;
};

dictionary JsonWebKey {
  // The following fields are defined in Section 3.1 of JSON Web Key
  DOMString kty;
  DOMString use;
  sequence<DOMString> key_ops;
  DOMString alg;

  // The following fields are defined in JSON Web Key Parameters Registration
  boolean ext;

  // The following fields are defined in Section 6 of JSON Web Algorithms
  DOMString crv;
  DOMString x;
  DOMString y;
  DOMString d;
  DOMString n;
  DOMString e;
  DOMString p;
  DOMString q;
  DOMString dp;
  DOMString dq;
  DOMString qi;
  sequence<RsaOtherPrimesInfo> oth;
  DOMString k;
};

// https://wicg.github.io/webcrypto-modern-algos/#subtlecrypto-interface-keyformat
// * For all existing symmetric algorithms in [webcrypto], "raw-secret"
//   acts as an alias of "raw".
// * For all existing asymmetric algorithms in [webcrypto], "raw-public"
//   acts as an alias of "raw".
// * In the deriveKey() method, in the import key step, "raw-secret"
//   must be used as the format instead of "raw".

enum KeyFormat { "raw-public", "raw-private", "raw-seed", "raw-secret", "raw", "spki", "pkcs8", "jwk" };

// https://wicg.github.io/webcrypto-modern-algos/#aead-params

dictionary AeadParams : Algorithm {
  required BufferSource iv;
  BufferSource additionalData;
  [EnforceRange] octet tagLength;
};

// https://wicg.github.io/webcrypto-modern-algos/#cshake-params

dictionary CShakeParams : Algorithm {
  required [EnforceRange] unsigned long length;
  BufferSource functionName;
  BufferSource customization;
};

// https://wicg.github.io/webcrypto-modern-algos/#argon2-params

dictionary Argon2Params : Algorithm {
  required BufferSource nonce;
  required [EnforceRange] unsigned long parallelism;
  required [EnforceRange] unsigned long memory;
  required [EnforceRange] unsigned long passes;
  [EnforceRange] octet version;
  BufferSource secretValue;
  BufferSource associatedData;
};
