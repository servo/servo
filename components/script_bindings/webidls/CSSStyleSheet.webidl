/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-cssstylesheet-interface
[Exposed=Window]
interface CSSStyleSheet : StyleSheet {
  // readonly attribute CSSRule? ownerRule;
  [Throws, SameObject] readonly attribute CSSRuleList cssRules;
  [Throws] unsigned long insertRule(DOMString rule, optional unsigned long index = 0);
  [Throws] undefined deleteRule(unsigned long index);
};

// https://drafts.csswg.org/cssom/#legacy-css-style-sheet-members
partial interface CSSStyleSheet {
  [Throws, SameObject] readonly attribute CSSRuleList rules;
  [Throws] long addRule(
      optional DOMString selector = "undefined",
      optional DOMString style = "undefined",
      optional unsigned long index
  );
  [Throws] undefined removeRule(optional unsigned long index = 0);
};
