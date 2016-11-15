/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-stylesheet-interface
[Exposed=Window]
interface StyleSheet {
  readonly attribute DOMString type_;
  readonly attribute DOMString? href;

  // readonly attribute (Element or ProcessingInstruction)? ownerNode;
  // readonly attribute StyleSheet? parentStyleSheet;
  readonly attribute DOMString? title;

  // [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  // attribute boolean disabled;
};
