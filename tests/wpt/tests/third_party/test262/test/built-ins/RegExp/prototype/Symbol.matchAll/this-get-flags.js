// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Regexp's flags
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    3. Return ? MatchAllIterator(R, string).

  MatchAllIterator ( R, O )
    [...]
    2. If ? IsRegExp(R) is true, then
      [...]
      b. Let flags be ? ToString(? Get(R, "flags"))
features: [Symbol.matchAll]
includes: [compareArray.js, compareIterator.js, regExpUtils.js]
---*/

var regexp = /\w/;
Object.defineProperty(regexp, 'flags', {
  value: 'g'
});
var str = 'a*b';

assert.compareIterator(regexp[Symbol.matchAll](str), [
  matchValidator(['a'], 0, str),
  matchValidator(['b'], 2, str)
]);
