/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* https://encoding.spec.whatwg.org/#interface-textencoder */
[Constructor, Exposed=(Window,Worker)]
interface TextEncoder {
   readonly attribute DOMString encoding;
   [NewObject]
   Uint8Array encode(optional USVString input = "");
};
