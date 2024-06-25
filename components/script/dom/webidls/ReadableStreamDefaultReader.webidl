/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#generic-reader-mixin-definition
// https://streams.spec.whatwg.org/#default-reader-class-definition

typedef (ReadableStreamDefaultReader or ReadableStreamBYOBReader) ReadableStreamReader;

interface mixin ReadableStreamGenericReader {
  readonly attribute Promise<undefined> closed;

  [NewObject]
  Promise<undefined> cancel(optional any reason);
};

[Exposed=*]
interface ReadableStreamDefaultReader {
  [Throws]
  constructor(ReadableStream stream);

  [NewObject]
  Promise<ReadableStreamReadResult> read();

  [Throws]
  undefined releaseLock();
};
ReadableStreamDefaultReader includes ReadableStreamGenericReader;


dictionary ReadableStreamReadResult {
 any value;
 boolean done;
};

