/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-stylesheet-interface
[Exposed=Window]
interface StyleSheet {
  readonly attribute DOMString type;
  readonly attribute DOMString? href;

  readonly attribute Element? ownerNode;
  [Unimplemented] readonly attribute StyleSheet? parentStyleSheet;
  readonly attribute DOMString? title;

  [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  attribute boolean disabled;
};

// https://drafts.csswg.org/cssom/#the-linkstyle-interface
interface mixin LinkStyle {
  readonly attribute StyleSheet? sheet;
};
