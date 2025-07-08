/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * For more information on this interface please see
 * https://encoding.spec.whatwg.org/#textdecoderstream
 */

[Exposed=*]
interface TextDecoderStream {
  constructor(optional DOMString label = "utf-8", optional TextDecoderOptions options = {});
};
TextDecoderStream includes TextDecoderCommon;
TextDecoderStream includes GenericTransformStream;