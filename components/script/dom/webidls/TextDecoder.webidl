/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this file is https://encoding.spec.whatwg.org/#interface-textdecoder
 *
 */

dictionary TextDecoderOptions {
  boolean fatal = false;
//  boolean ignoreBOM = false;
};

/*dictionary TextDecodeOptions {
  boolean stream = false;
};*/

[Constructor(optional DOMString label = "utf-8", optional TextDecoderOptions options)/*,
 Exposed=Window,Worker*/]
interface TextDecoder {
  readonly attribute DOMString encoding;
  readonly attribute boolean fatal;
  readonly attribute boolean ignoreBOM;
  // FIXME: decode should return a USVString instead, and ArrayBuffer should really be BufferSource
  [Throws]
  DOMString decode(optional ArrayBuffer input/*, optional TextDecodeOptions options*/);
};
