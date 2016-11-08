/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlhyperlinkelementutils
[NoInterfaceObject]
interface HTMLHyperlinkElementUtils {
//  stringifier attribute USVString href;
             attribute USVString href;
    readonly attribute USVString origin;
             attribute USVString protocol;
             attribute USVString username;
             attribute USVString password;
             attribute USVString host;
             attribute USVString hostname;
             attribute USVString port;
             attribute USVString pathname;
             attribute USVString search;
             attribute USVString hash;

  // Adding a separate stringifier method until
  // https://github.com/servo/servo/issues/7590 adds attribute stringifier
  // support.
  stringifier;
};
