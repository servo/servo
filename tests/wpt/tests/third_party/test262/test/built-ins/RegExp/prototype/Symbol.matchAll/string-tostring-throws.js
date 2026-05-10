// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: String coercion of string parameter
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    3. Return ? MatchAllIterator(R, string).

  MatchAllIterator ( R, O )
    1. Let S be ? ToString(O).
features: [Symbol.matchAll]
---*/

var obj = {
  valueOf() {
    throw new Test262Error('This method should not be invoked.');
  },
  toString() {
    throw new Test262Error('toString invoked');
  }
};

assert.throws(Test262Error, function() {
  /toString value/[Symbol.matchAll](obj);
});
