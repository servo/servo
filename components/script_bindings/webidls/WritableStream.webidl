/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#writablestream

[Exposed=*] // [Transferable] - See Bug 1562065
interface WritableStream {
  [Throws]
  constructor(optional object underlyingSink, optional QueuingStrategy strategy = {});

  readonly attribute boolean locked;

  Promise<undefined> abort(optional any reason);
  Promise<undefined> close();

  [Throws]
  WritableStreamDefaultWriter getWriter();
};
