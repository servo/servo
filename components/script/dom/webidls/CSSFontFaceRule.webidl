/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/css-fonts/#cssfontfacerule is unfortunately not web-compatible:
// https://github.com/w3c/csswg-drafts/issues/825

// https://www.w3.org/TR/2000/REC-DOM-Level-2-Style-20001113/css.html#CSS-CSSFontFaceRule ,
// plus extended attributes matching CSSStyleRule
[Exposed=Window]
interface CSSFontFaceRule : CSSRule {
  // [SameObject, PutForwards=cssText] readonly attribute CSSStyleDeclaration style;
};

