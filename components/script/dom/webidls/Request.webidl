/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://fetch.spec.whatwg.org/#request-class

typedef (Request or USVString) RequestInfo;

[Exposed=(Window,Worker)]
interface Request {
  [Throws] constructor(RequestInfo input, optional RequestInit init = {});
  readonly attribute ByteString method;
  readonly attribute USVString url;
  [SameObject] readonly attribute Headers headers;

  readonly attribute RequestDestination destination;
  readonly attribute USVString referrer;
  readonly attribute ReferrerPolicy referrerPolicy;
  readonly attribute RequestMode mode;
  readonly attribute RequestCredentials credentials;
  readonly attribute RequestCache cache;
  readonly attribute RequestRedirect redirect;
  readonly attribute DOMString integrity;

  [NewObject, Throws] Request clone();
};

Request includes Body;

dictionary RequestInit {
  ByteString method;
  HeadersInit headers;
  BodyInit? body;
  USVString referrer;
  ReferrerPolicy referrerPolicy;
  RequestMode mode;
  RequestCredentials credentials;
  RequestCache cache;
  RequestRedirect redirect;
  DOMString integrity;
  any window; // can only be set to null
};

enum RequestDestination {
  "",
  "audio",
  "document",
  "embed",
  "font",
  "frame",
  "iframe",
  "image",
  "json",
  "manifest",
  "object",
  "report",
  "script",
  "sharedworker",
  "style",
  "track",
  "video",
  "worker",
  "xslt"
};

enum RequestMode {
  "navigate",
  "same-origin",
  "no-cors",
  "cors"
};

enum RequestCredentials {
  "omit",
  "same-origin",
  "include"
};

enum RequestCache {
  "default",
  "no-store",
  "reload",
  "no-cache",
  "force-cache",
  "only-if-cached"
};

enum RequestRedirect {
  "follow",
  "error",
  "manual"
};

enum ReferrerPolicy {
  "",
  "no-referrer",
  "no-referrer-when-downgrade",
  "origin",
  "origin-when-cross-origin",
  "unsafe-url",
  "same-origin",
  "strict-origin",
  "strict-origin-when-cross-origin"
};
