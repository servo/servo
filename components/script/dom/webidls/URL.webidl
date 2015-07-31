/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://url.spec.whatwg.org/#url
[Constructor(USVString url, optional USVString base)/*,
 Exposed=(Window,Worker)*/]
interface URL {
  static USVString domainToASCII(USVString domain);
  // static USVString domainToUnicode(USVString domain);
};
URL implements URLUtils;
