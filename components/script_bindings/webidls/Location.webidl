/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#location
[Exposed=Window, LegacyUnforgeable] interface Location {
  [Throws, CrossOriginWritable]
        stringifier attribute USVString href;
  [Throws] readonly attribute USVString origin;
  [Throws]          attribute USVString protocol;
  [Throws]          attribute USVString host;
  [Throws]          attribute USVString hostname;
  [Throws]          attribute USVString port;
  [Throws]          attribute USVString pathname;
  [Throws]          attribute USVString search;
  [Throws]          attribute USVString hash;

  [Throws] undefined assign(USVString url);
  [Throws, CrossOriginCallable]
           undefined replace(USVString url);
  [Throws] undefined reload();

  //[SameObject] readonly attribute USVString[] ancestorOrigins;
};
