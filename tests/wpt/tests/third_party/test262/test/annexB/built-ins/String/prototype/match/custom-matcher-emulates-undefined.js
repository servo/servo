// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.match
description: >
  [[IsHTMLDDA]] object as @@match method gets called.
info: |
  String.prototype.match ( regexp )

  [...]
  2. If regexp is neither undefined nor null, then
    a. Let matcher be ? GetMethod(regexp, @@match).
    b. If matcher is not undefined, then
      i. Return ? Call(matcher, regexp, « O »).
features: [Symbol.match, IsHTMLDDA]
---*/

var regexp = $262.IsHTMLDDA;
var matcherGets = 0;
Object.defineProperty(regexp, Symbol.match, {
  get: function() {
    matcherGets += 1;
    return regexp;
  },
  configurable: true,
});

assert.sameValue("".match(regexp), null);
assert.sameValue(matcherGets, 1);
