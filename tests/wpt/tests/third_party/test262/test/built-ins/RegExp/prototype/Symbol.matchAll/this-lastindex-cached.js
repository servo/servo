// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Verify regexp's lastIndex is cached
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    3. Return ? MatchAllIterator(R, string).

  MatchAllIterator ( R, O )
    [...]
    2. If ? IsRegExp(R) is true, then
      [...]
      f. Let lastIndex be ? ToLength(? Get(R, "lastIndex")).
      g. Perform ? Set(matcher, "lastIndex", lastIndex, true).
features: [Symbol.matchAll]
includes: [compareArray.js, compareIterator.js, regExpUtils.js]
---*/

var regexp = /./g;
regexp.lastIndex = {
  valueOf() {
    return 2;
  }
};
var str = 'abcd';
var iter = regexp[Symbol.matchAll](str);

// Verify lastIndex is cached at the time of calling @@matchAll
regexp.lastIndex = 0;

assert.compareIterator(iter, [
  matchValidator(['c'], 2, str),
  matchValidator(['d'], 3, str)
]);
