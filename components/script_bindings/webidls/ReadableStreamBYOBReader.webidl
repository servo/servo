/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#byob-reader-class-definition

[Exposed=*]
interface ReadableStreamBYOBReader {
  [Throws]
  constructor(ReadableStream stream);

  [NewObject]
  Promise<ReadableStreamReadResult> read(ArrayBufferView view,
      optional ReadableStreamBYOBReaderReadOptions options = {}
  );

  [Throws]
  undefined releaseLock();
};
ReadableStreamBYOBReader includes ReadableStreamGenericReader;

dictionary ReadableStreamBYOBReaderReadOptions {
  [EnforceRange] unsigned long long min = 1;
};
