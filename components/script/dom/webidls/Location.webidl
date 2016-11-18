/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#location
[Exposed=(Window,Worker), Unforgeable] interface Location {
  /*stringifier*/ attribute USVString href;
  readonly attribute USVString origin;
           attribute USVString protocol;
           attribute USVString host;
           attribute USVString hostname;
           attribute USVString port;
           attribute USVString pathname;
           attribute USVString search;
           attribute USVString hash;

  [Throws]
  void assign(USVString url);
  [Throws]
  void replace(USVString url);
  void reload();

  //[SameObject] readonly attribute USVString[] ancestorOrigins;

  // This is only doing as well as gecko right now.
  // https://github.com/servo/servo/issues/7590 is on file for
  // adding attribute stringifier support.
  stringifier;
};
