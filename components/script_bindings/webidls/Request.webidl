/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://fetch.spec.whatwg.org/#request-class

typedef (Request or USVString) RequestInfo;

// https://fetch.spec.whatwg.org/#request
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
  readonly attribute boolean keepalive;
  // readonly attribute boolean isReloadNavigation;
  // readonly attribute boolean isHistoryNavigation;
  readonly attribute AbortSignal signal;
  // readonly attribute RequestDuplex duplex;

  [NewObject, Throws] Request clone();
};

Request includes Body;

// https://fetch.spec.whatwg.org/#requestinit
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
  // boolean keepalive;
  AbortSignal? signal;
  // RequestDuplex duplex;
  // RequestPriority priority;
  any window; // can only be set to null
};

// https://fetch.spec.whatwg.org/#requestdestination
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

// https://fetch.spec.whatwg.org/#requestmode
enum RequestMode {
  "navigate",
  "same-origin",
  "no-cors",
  "cors"
};

// https://fetch.spec.whatwg.org/#requestcredentials
enum RequestCredentials {
  "omit",
  "same-origin",
  "include"
};

// https://fetch.spec.whatwg.org/#requestcache
enum RequestCache {
  "default",
  "no-store",
  "reload",
  "no-cache",
  "force-cache",
  "only-if-cached"
};

// https://fetch.spec.whatwg.org/#requestredirect
enum RequestRedirect {
  "follow",
  "error",
  "manual"
};

// https://w3c.github.io/webappsec-referrer-policy/#enumdef-referrerpolicy
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
