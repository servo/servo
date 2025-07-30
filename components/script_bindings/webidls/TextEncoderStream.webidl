/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * For more information on this interface please see
 * https://encoding.spec.whatwg.org/#textencoderstream
 */

[Exposed=*]
interface TextEncoderStream {
  [Throws] constructor();
};
TextEncoderStream includes TextEncoderCommon;
TextEncoderStream includes GenericTransformStream;