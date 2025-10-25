/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://compression.spec.whatwg.org/#enumdef-compressionformat
enum CompressionFormat {
  "deflate",
  "deflate-raw",
  "gzip",
};

// https://compression.spec.whatwg.org/#compressionstream
[Exposed=*]
interface CompressionStream {
  [Throws] constructor(CompressionFormat format);
};
CompressionStream includes GenericTransformStream;
