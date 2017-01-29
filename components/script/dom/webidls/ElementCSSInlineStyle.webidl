/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//http://dev.w3.org/csswg/cssom/#elementcssinlinestyle

[NoInterfaceObject, Exposed=Window]
interface ElementCSSInlineStyle {
  [SameObject/*, PutForwards=cssText*/] readonly attribute CSSStyleDeclaration style;
};
