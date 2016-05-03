/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://dev.w3.org/2006/webapi/FileAPI/#dfn-file

// [Constructor(sequence<(Blob or DOMString or ArrayBufferView or ArrayBuffer)> fileBits,
//              [EnsureUTF16] DOMString fileName, optional FilePropertyBag options)]
interface File : Blob {

  readonly attribute DOMString name;
  // readonly attribute Date lastModifiedDate;

};
