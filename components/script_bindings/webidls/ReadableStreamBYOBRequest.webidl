/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#rs-byob-request-class-definition

[Exposed=*]
interface ReadableStreamBYOBRequest {
  readonly attribute ArrayBufferView? view;

  [Throws]
  undefined respond([EnforceRange] unsigned long long bytesWritten);
  [Throws]
  undefined respondWithNewView(ArrayBufferView view);
};
