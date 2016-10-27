/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://url.spec.whatwg.org/#url
[Constructor(USVString url, optional USVString base), Exposed=(Window,Worker)]
interface URL {
  [SetterThrows]
  /*stringifier*/ attribute USVString href;
  readonly attribute USVString origin;
           attribute USVString protocol;
           attribute USVString username;
           attribute USVString password;
           attribute USVString host;
           attribute USVString hostname;
           attribute USVString port;
           attribute USVString pathname;
           attribute USVString search;
  readonly attribute URLSearchParams searchParams;
           attribute USVString hash;

  // https://w3c.github.io/FileAPI/#creating-revoking
  static DOMString createObjectURL(Blob blob);
  // static DOMString createFor(Blob blob);
  static void revokeObjectURL(DOMString url);

  // This is only doing as well as gecko right now.
  // https://github.com/servo/servo/issues/7590 is on file for
  // adding attribute stringifier support.
  stringifier;
};
