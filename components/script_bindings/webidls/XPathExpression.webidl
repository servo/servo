/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#interface-xpathexpression
[Exposed=Window, Pref="dom_xpath_enabled"]
interface XPathExpression {
  // XPathResult.ANY_TYPE = 0
  [Throws] XPathResult evaluate(
    Node contextNode,
    optional unsigned short type = 0,
    optional XPathResult? result = null
  );
};
