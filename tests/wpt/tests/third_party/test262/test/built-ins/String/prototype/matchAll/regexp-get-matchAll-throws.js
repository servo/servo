// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors when calling @@matchAll
info: |
  String.prototype.matchAll ( regexp )
    [...]
    2. If regexp is neither undefined nor null, then
      a. Let matcher be ? GetMethod(regexp, @@matchAll).
      b. If matcher is not undefined, then
        i. Return ? Call(matcher, regexp, « O »).
features: [Symbol.matchAll, String.prototype.matchAll]
---*/

var regexp = /./g;
Object.defineProperty(regexp, Symbol.matchAll, {
  get() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  ''.matchAll(regexp);
});
