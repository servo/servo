/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/FileAPI/#blob

[Constructor(optional sequence<BlobPart> blobParts,
  optional BlobPropertyBag options),
 Exposed=(Window,Worker)]
interface Blob {

  readonly attribute unsigned long long size;
  readonly attribute DOMString type;

  // slice Blob into byte-ranged chunks
  Blob slice([Clamp] optional long long start,
             [Clamp] optional long long end,
             optional DOMString contentType);
};

dictionary BlobPropertyBag {
  DOMString type = "";
};

typedef (/*ArrayBuffer or ArrayBufferView or */Blob or DOMString) BlobPart;
