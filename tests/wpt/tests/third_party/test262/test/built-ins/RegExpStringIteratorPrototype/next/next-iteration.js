// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Iterates over the first match
info: |
  %RegExpStringIteratorPrototype%.next ( )
    [...]
    4. If O.[[Done]] is true, then
      a. Return ! reateIterResultObject(undefined, true).
    [...]
    9. Let match be ? RegExpExec(R, S).
    10. If match is null, then
      a. Set O.[[Done]] to true.
      b. Return ! CreateIterResultObject(undefined, true).
    11. Else,
      a. If global is true,
        [...]
      b. Else,
        i. Set O.[[Done]] to true.
        ii. Return ! CreateIterResultObject(match, false).
features: [Symbol.matchAll]
includes: [compareArray.js, compareIterator.js, regExpUtils.js]
---*/

var regexp = /\w/;
var str = '*a*b';
var iter = regexp[Symbol.matchAll](str);

assert.compareIterator(iter, [
  matchValidator(['a'], 1, str)
]);

// Verifies %RegExpStringIteratorPrototype%.next() step 4
var result = iter.next();
assert.sameValue(result.value, undefined);
assert(result.done);
