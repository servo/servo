/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/css-animations/#interface-csskeyframesrule
[Exposed=Window]
interface CSSKeyframesRule : CSSRule {
  [SetterThrows]
           attribute DOMString   name;
  readonly attribute CSSRuleList cssRules;

  void            appendRule(DOMString rule);
  void            deleteRule(DOMString select);
  CSSKeyframeRule? findRule(DOMString select);
};
