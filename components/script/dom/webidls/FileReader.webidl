/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 // http://dev.w3.org/2006/webapi/FileAPI/#APIASynch

typedef (DOMString or object) FileReaderResult;
[Constructor, Exposed=(Window,Worker)]
interface FileReader: EventTarget {

  // async read methods
  [Throws]
  void readAsArrayBuffer(Blob blob);
  [Throws]
  void readAsText(Blob blob, optional DOMString label);
  [Throws]
  void readAsDataURL(Blob blob);

  void abort();

  // states
  const unsigned short EMPTY = 0;
  const unsigned short LOADING = 1;
  const unsigned short DONE = 2;
  readonly attribute unsigned short readyState;

  // File or Blob data
  readonly attribute FileReaderResult? result;

  readonly attribute DOMException? error;

  // event handler attributes
  attribute EventHandler onloadstart;
  attribute EventHandler onprogress;
  attribute EventHandler onload;
  attribute EventHandler onabort;
  attribute EventHandler onerror;
  attribute EventHandler onloadend;

};
