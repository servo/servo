/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-cssstylesheet-interface
[Exposed=Window]
interface CSSStyleSheet : StyleSheet {
  // readonly attribute CSSRule? ownerRule;
  [Throws, SameObject] readonly attribute CSSRuleList cssRules;
  [Throws] unsigned long insertRule(DOMString rule, unsigned long index);
  [Throws] void deleteRule(unsigned long index);
};
