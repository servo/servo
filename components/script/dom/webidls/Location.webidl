/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#location
[Exposed=Window, Unforgeable] interface Location {
  /*stringifier*/ [Throws] attribute USVString href;
  [Throws] readonly attribute USVString origin;
  [Throws]          attribute USVString protocol;
  [Throws]          attribute USVString host;
  [Throws]          attribute USVString hostname;
  [Throws]          attribute USVString port;
  [Throws]          attribute USVString pathname;
  [Throws]          attribute USVString search;
  [Throws]          attribute USVString hash;

  [Throws] void assign(USVString url);
  [Throws] void replace(USVString url);
  [Throws] void reload();

  //[SameObject] readonly attribute USVString[] ancestorOrigins;

  // This is only doing as well as gecko right now.
  // https://github.com/servo/servo/issues/7590 is on file for
  // adding attribute stringifier support.
  [Throws] stringifier;
};
