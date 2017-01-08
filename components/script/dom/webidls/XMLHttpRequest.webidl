/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://xhr.spec.whatwg.org/#interface-xmlhttprequest
 *
 * To the extent possible under law, the editor has waived all copyright
 * and related or neighboring rights to this work. In addition, as of 1 May 2014,
 * the editor has made this specification available under the Open Web Foundation
 * Agreement Version 1.0, which is available at
 * http://www.openwebfoundation.org/legal/the-owf-1-0-agreements/owfa-1-0.
 */

// https://fetch.spec.whatwg.org/#bodyinit
typedef (Blob or /*BufferSource or */ FormData or DOMString or URLSearchParams) BodyInit;

enum XMLHttpRequestResponseType {
  "",
  "arraybuffer",
  "blob",
  "document",
  "json",
  "text"
};

[Constructor, Exposed=(Window,Worker)]
interface XMLHttpRequest : XMLHttpRequestEventTarget {
  // event handler
  attribute EventHandler onreadystatechange;

  // states
  const unsigned short UNSENT = 0;
  const unsigned short OPENED = 1;
  const unsigned short HEADERS_RECEIVED = 2;
  const unsigned short LOADING = 3;
  const unsigned short DONE = 4;
  readonly attribute unsigned short readyState;

  // request
  [Throws]
  void open(ByteString method, USVString url);
  [Throws]
  void open(ByteString method, USVString url, boolean async,
            optional USVString? username = null,
            optional USVString? password = null);

  [Throws]
  void setRequestHeader(ByteString name, ByteString value);
  [SetterThrows]
           attribute unsigned long timeout;
  [SetterThrows]
           attribute boolean withCredentials;
  readonly attribute XMLHttpRequestUpload upload;
  [Throws]
  void send(optional (Document or BodyInit)? data = null);
  void abort();

  // response
  readonly attribute USVString responseURL;
  readonly attribute unsigned short status;
  readonly attribute ByteString statusText;
  ByteString? getResponseHeader(ByteString name);
  ByteString getAllResponseHeaders();
  [Throws]
  void overrideMimeType(DOMString mime);
  [SetterThrows]
           attribute XMLHttpRequestResponseType responseType;
  readonly attribute any response;
  [Throws]
  readonly attribute USVString responseText;
  [Throws, Exposed=Window] readonly attribute Document? responseXML;
};
