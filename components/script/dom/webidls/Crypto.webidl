/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#crypto-interface
 *
 */

[Exposed=(Window,Worker)]
interface mixin GlobalCrypto {
  readonly attribute Crypto crypto;
};

Window includes GlobalCrypto;
WorkerGlobalScope includes GlobalCrypto;

[Exposed=(Window,Worker)]
interface Crypto {
  //readonly attribute SubtleCrypto subtle;
  [Throws]
  ArrayBufferView getRandomValues(ArrayBufferView array);
};
