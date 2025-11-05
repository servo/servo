/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://url.spec.whatwg.org/#url
[Exposed=(Window,Worker),
 LegacyWindowAlias=webkitURL]
interface URL {
  [Throws] constructor(USVString url, optional USVString base);

  static URL? parse(USVString url, optional USVString base);
  static boolean canParse(USVString url, optional USVString base);

  [SetterThrows]
  stringifier attribute USVString href;
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
  static DOMString createObjectURL((Blob or MediaSource) blob);
  // static DOMString createFor(Blob blob);
  static undefined revokeObjectURL(DOMString url);

  USVString toJSON();
};
