// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.matchall
description: >
  [[IsHTMLDDA]] object as @@matchAll method gets called.
info: |
  String.prototype.matchAll ( regexp )

  [...]
  2. If regexp is neither undefined nor null, then
    [...]
    c. Let matcher be ? GetMethod(regexp, @@matchAll).
    d. If matcher is not undefined, then
      i. Return ? Call(matcher, regexp, « O »).
features: [Symbol.matchAll, String.prototype.matchAll, IsHTMLDDA]
---*/

var regexp = $262.IsHTMLDDA;
var matcherGets = 0;
Object.defineProperty(regexp, Symbol.matchAll, {
  get: function() {
    matcherGets += 1;
    return regexp;
  },
  configurable: true,
});

assert.sameValue("".matchAll(regexp), null);
assert.sameValue(matcherGets, 1);
