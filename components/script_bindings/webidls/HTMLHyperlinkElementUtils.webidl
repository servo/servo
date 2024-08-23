/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlhyperlinkelementutils
interface mixin HTMLHyperlinkElementUtils {
  [CEReactions]
  stringifier attribute USVString href;
  readonly attribute USVString origin;
  [CEReactions]
           attribute USVString protocol;
  [CEReactions]
           attribute USVString username;
  [CEReactions]
           attribute USVString password;
  [CEReactions]
           attribute USVString host;
  [CEReactions]
           attribute USVString hostname;
  [CEReactions]
           attribute USVString port;
  [CEReactions]
           attribute USVString pathname;
  [CEReactions]
           attribute USVString search;
  [CEReactions]
           attribute USVString hash;
};
