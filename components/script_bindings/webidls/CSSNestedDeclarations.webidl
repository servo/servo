/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/css-nesting-1/#the-cssnestrule
 */

[Exposed=Window]
interface CSSNestedDeclarations : CSSRule {
  // CSSStyleDeclaration instead of CSSStyleProperties for now, see #36260.
  [SameObject, PutForwards=cssText] readonly attribute CSSStyleDeclaration style;
};
