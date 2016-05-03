/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://dev.w3.org/2006/webapi/FileAPI/#dfn-Blob
[Constructor(optional sequence<(/*ArrayBuffer or ArrayBufferView or */Blob or DOMString)> blobParts,
  optional BlobPropertyBag options),
 Exposed=Window/*,Worker*/]
interface Blob {

  readonly attribute unsigned long long size;
  readonly attribute DOMString type;
  readonly attribute boolean isClosed;

  //slice Blob into byte-ranged chunks

  Blob slice([Clamp] optional long long start,
             [Clamp] optional long long end,
             optional DOMString contentType);
  void close();

};

dictionary BlobPropertyBag {

  DOMString type = "";

};
