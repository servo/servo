/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/FileAPI/#blob

[Exposed=(Window,Worker)]
interface Blob {
  [Throws] constructor(optional sequence<BlobPart> blobParts,
    optional BlobPropertyBag options = {});

  readonly attribute unsigned long long size;
  readonly attribute DOMString type;

  // slice Blob into byte-ranged chunks
  Blob slice(optional [Clamp] long long start,
             optional [Clamp] long long end,
             optional DOMString contentType);

  [NewObject] object stream();
  [NewObject] Promise<DOMString> text();
  [NewObject] Promise<ArrayBuffer> arrayBuffer();
};

dictionary BlobPropertyBag {
  DOMString type = "";
};

typedef (ArrayBuffer or ArrayBufferView or Blob or DOMString) BlobPart;
