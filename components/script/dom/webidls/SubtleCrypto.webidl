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

[SecureContext,Exposed=(Window,Worker)]
interface SubtleCrypto {
  // Promise<any> encrypt(AlgorithmIdentifier algorithm,
  //                      CryptoKey key,
  //                      BufferSource data);
  // Promise<any> decrypt(AlgorithmIdentifier algorithm,
  //                      CryptoKey key,
  //                      BufferSource data);
  // Promise<any> sign(AlgorithmIdentifier algorithm,
  //                   CryptoKey key,
  //                   BufferSource data);
  // Promise<any> verify(AlgorithmIdentifier algorithm,
  //                     CryptoKey key,
  //                     BufferSource signature,
  //                     BufferSource data);
  // Promise<any> digest(AlgorithmIdentifier algorithm,
  //                     BufferSource data);

  Promise<any> generateKey(AlgorithmIdentifier algorithm,
                          boolean extractable,
                          sequence<KeyUsage> keyUsages );
  // Promise<any> deriveKey(AlgorithmIdentifier algorithm,
  //                        CryptoKey baseKey,
  //                        AlgorithmIdentifier derivedKeyType,
  //                        boolean extractable,
  //                        sequence<KeyUsage> keyUsages );
  // Promise<ArrayBuffer> deriveBits(AlgorithmIdentifier algorithm,
  //                         CryptoKey baseKey,
  //                         optional unsigned long? length = null);

  // Promise<CryptoKey> importKey(KeyFormat format,
  //                        (BufferSource or JsonWebKey) keyData,
  //                        AlgorithmIdentifier algorithm,
  //                        boolean extractable,
  //                        sequence<KeyUsage> keyUsages );
  // Promise<any> exportKey(KeyFormat format, CryptoKey key);

  // Promise<any> wrapKey(KeyFormat format,
  //                      CryptoKey key,
  //                      CryptoKey wrappingKey,
  //                      AlgorithmIdentifier wrapAlgorithm);
  // Promise<CryptoKey> unwrapKey(KeyFormat format,
  //                        BufferSource wrappedKey,
  //                        CryptoKey unwrappingKey,
  //                        AlgorithmIdentifier unwrapAlgorithm,
  //                        AlgorithmIdentifier unwrappedKeyAlgorithm,
  //                        boolean extractable,
  //                        sequence<KeyUsage> keyUsages );
};

// AES_CBC
dictionary AesCbcParams : Algorithm {
  required BufferSource iv;
};

dictionary AesKeyGenParams : Algorithm {
  required [EnforceRange] unsigned short length;
};

dictionary AesDerivedKeyParams : Algorithm {
  required [EnforceRange] unsigned short length;
};
