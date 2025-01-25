/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webcrypto/#subtlecrypto-interface

typedef (object or DOMString) AlgorithmIdentifier;

typedef AlgorithmIdentifier HashAlgorithmIdentifier;

dictionary Algorithm {
  required DOMString name;
};

dictionary KeyAlgorithm {
  required DOMString name;
};

enum KeyFormat { "raw", "spki", "pkcs8", "jwk" };

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

// AES shared
dictionary AesKeyAlgorithm : KeyAlgorithm {
  required unsigned short length;
};

dictionary AesKeyGenParams : Algorithm {
  required [EnforceRange] unsigned short length;
};

dictionary AesDerivedKeyParams : Algorithm {
  required [EnforceRange] unsigned short length;
};

// AES_CBC
dictionary AesCbcParams : Algorithm {
  required BufferSource iv;
};

// AES_CTR
dictionary AesCtrParams : Algorithm {
  required BufferSource counter;
  required [EnforceRange] octet length;
};

// https://w3c.github.io/webcrypto/#aes-gcm-params
dictionary AesGcmParams : Algorithm {
  required BufferSource iv;
  BufferSource additionalData;
  [EnforceRange] octet tagLength;
};

// https://w3c.github.io/webcrypto/#dfn-HmacImportParams
dictionary HmacImportParams : Algorithm {
  required HashAlgorithmIdentifier hash;
  [EnforceRange] unsigned long length;
};

// https://w3c.github.io/webcrypto/#dfn-HmacKeyAlgorithm
dictionary HmacKeyAlgorithm : KeyAlgorithm {
  required KeyAlgorithm hash;
  required unsigned long length;
};

// https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams
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

// JWK
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
