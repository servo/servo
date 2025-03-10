/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#cssimportrule
[Exposed=Window]
interface CSSImportRule : CSSRule {
  [Unimplemented] readonly attribute DOMString href;
  [Unimplemented, SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  [Unimplemented, SameObject] readonly attribute CSSStyleSheet styleSheet;
  readonly attribute DOMString? layerName;
};
