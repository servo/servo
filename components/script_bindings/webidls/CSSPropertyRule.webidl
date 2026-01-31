/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://drafts.css-houdini.org/css-properties-values-api/#the-css-property-rule-interface
 */


[Exposed=Window]
interface CSSPropertyRule : CSSRule {
  readonly attribute CSSOMString name;
  readonly attribute CSSOMString syntax;
  readonly attribute boolean inherits;
  readonly attribute CSSOMString? initialValue;
};
