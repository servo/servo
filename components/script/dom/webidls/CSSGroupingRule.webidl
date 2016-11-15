/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom/#the-cssgroupingrule-interface
[Exposed=Window]
interface CSSGroupingRule : CSSRule {
  // [SameObject] readonly attribute CSSRuleList cssRules;
  // unsigned long insertRule(DOMString rule, unsigned long index);
  // void deleteRule(unsigned long index);
};

