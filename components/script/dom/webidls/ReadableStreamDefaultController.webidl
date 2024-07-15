/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#rs-default-controller-class-definition

[Exposed=*]
interface ReadableStreamDefaultController {
  readonly attribute unrestricted double? desiredSize;

  [Throws]
  undefined close();
  [Throws]
  undefined enqueue(optional any chunk);
  [Throws]
  undefined error(optional any e);
};
