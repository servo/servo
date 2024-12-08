/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/webcrypto/#crypto-interface
 *
 */

partial interface mixin WindowOrWorkerGlobalScope {
  [SameObject] readonly attribute Crypto crypto;
};

[Exposed=(Window,Worker)]
interface Crypto {
  [SecureContext, Pref="dom_crypto_subtle_enabled"] readonly attribute SubtleCrypto subtle;
  [Throws] ArrayBufferView getRandomValues(ArrayBufferView array);
  [SecureContext] DOMString randomUUID();
};
