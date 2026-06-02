/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#rbs-controller-class-definition

[Exposed=*]
interface ReadableByteStreamController {
  [Throws] // Throws on OOM
  readonly attribute ReadableStreamBYOBRequest? byobRequest;
  readonly attribute unrestricted double? desiredSize;

  [Throws]
  undefined close();
  [Throws]
  undefined enqueue(ArrayBufferView chunk);
  [Throws]
  undefined error(optional any e);
};
