/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/FileAPI/#FileReaderSync

[Exposed=Worker]
interface FileReaderSync {
  [Throws] constructor();
  // Synchronously return strings

  [Throws]
  ArrayBuffer readAsArrayBuffer(Blob blob);
  [Throws]
  DOMString readAsBinaryString(Blob blob);
  [Throws]
  DOMString readAsText(Blob blob, optional DOMString label);
  [Throws]
  DOMString readAsDataURL(Blob blob);
};
