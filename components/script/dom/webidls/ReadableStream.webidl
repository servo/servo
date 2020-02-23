/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is: https://streams.spec.whatwg.org/#rs-class
 */

[Exposed=(Window,Worker)]
interface ReadableStream {
  constructor(object underlyingSource, Function size, HighWatermark highWaterMark, object proto);
  [Throws] Promise<DOMString> cancel(DOMString reason);
  [Throws] object getReader();
  readonly attribute boolean locked;
};

typedef double HighWatermark;
