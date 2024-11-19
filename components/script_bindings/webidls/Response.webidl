/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://fetch.spec.whatwg.org/#response-class

 [Exposed=(Window,Worker)]
interface Response {
  [Throws] constructor(optional BodyInit? body = null, optional ResponseInit init = {});
  [NewObject] static Response error();
  [NewObject, Throws] static Response redirect(USVString url, optional unsigned short status = 302);

  readonly attribute ResponseType type;

  readonly attribute USVString url;
  readonly attribute boolean redirected;
  readonly attribute unsigned short status;
  readonly attribute boolean ok;
  readonly attribute ByteString statusText;
  [SameObject] readonly attribute Headers headers;
  // readonly attribute ReadableStream? body;
  // [SameObject] readonly attribute Promise<Headers> trailer;

  [NewObject, Throws] Response clone();
};
Response includes Body;

dictionary ResponseInit {
  unsigned short status = 200;
  ByteString statusText = "";
  HeadersInit headers;
};

enum ResponseType { "basic", "cors", "default", "error", "opaque", "opaqueredirect" };

// typedef (BodyInit or ReadableStream) ResponseBodyInit;
