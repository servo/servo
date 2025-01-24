/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#mixin-xpathevaluatorbase
interface mixin XPathEvaluatorBase {
  [NewObject, Throws, Pref="dom_xpath_enabled"] XPathExpression createExpression(
    DOMString expression,
    optional XPathNSResolver? resolver = null
  );
  Node createNSResolver(Node nodeResolver); // legacy
  // XPathResult.ANY_TYPE = 0
  [Throws, Pref="dom_xpath_enabled"] XPathResult evaluate(
    DOMString expression,
    Node contextNode,
    optional XPathNSResolver? resolver = null,
    optional unsigned short type = 0,
    optional XPathResult? result = null
  );
};

Document includes XPathEvaluatorBase;

// https://dom.spec.whatwg.org/#interface-xpathevaluator
[Exposed=Window, Pref="dom_xpath_enabled"]
interface XPathEvaluator {
  constructor();
};

XPathEvaluator includes XPathEvaluatorBase;
