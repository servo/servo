/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#rs-class
[Exposed=(Window,Worker)]
interface ReadableStream {
  constructor(object underlyingSource, optional object queuingStrategy);
  [Throws] Promise<DOMString> cancel(DOMString reason);
  [Throws] object getReader();
  [Throws] /*FrozenArray<ReadableStream>*/any tee();
  readonly attribute boolean locked;
};

