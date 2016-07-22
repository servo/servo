/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/FileAPI/#file

[Constructor(sequence<BlobPart> fileBits,
            DOMString fileName,
            optional FilePropertyBag options),
 Exposed=(Window,Worker)]
interface File : Blob {
  readonly attribute DOMString name;
  readonly attribute long long lastModified;
};

dictionary FilePropertyBag : BlobPropertyBag {
  long long lastModified;
};
