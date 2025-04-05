/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-cssnamespacerule-interface
[Exposed=Window]
interface CSSNamespaceRule : CSSRule {
  readonly attribute DOMString namespaceURI;
  readonly attribute DOMString prefix;
};
