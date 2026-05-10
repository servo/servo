// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlastindex
description: >
  Return -1 if predicate always returns a boolean false value.
info: |
  Array.prototype.findLastIndex ( predicate[ , thisArg ] )

  ...
  5. Repeat, while k ≥ 0,
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    ...
  6. Return -1.
features: [Symbol, array-find-from-last]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var called = 0;

var result = arr.findLastIndex(function() {
  called++;
  return false;
});

assert.sameValue(called, 3, 'predicate was called three times');
assert.sameValue(result, -1);

result = arr.findLastIndex(function() {
  return '';
});
assert.sameValue(result, -1, 'coerced string');

result = arr.findLastIndex(function() {
  return undefined;
});
assert.sameValue(result, -1, 'coerced undefined');

result = arr.findLastIndex(function() {
  return null;
});
assert.sameValue(result, -1, 'coerced null');

result = arr.findLastIndex(function() {
  return 0;
});
assert.sameValue(result, -1, 'coerced 0');

result = arr.findLastIndex(function() {
  return NaN;
});
assert.sameValue(result, -1, 'coerced NaN');
