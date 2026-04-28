// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlast
description: >
  Return undefined if predicate always returns a boolean false value.
info: |
  Array.prototype.findLast ( predicate[ , thisArg ] )

  ...
  5. Repeat, while k ≥ 0,
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    ...
  6. Return undefined.
features: [Symbol, array-find-from-last]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var called = 0;

var result = arr.findLast(function() {
  called++;
  return false;
});

assert.sameValue(called, 3, 'predicate was called three times');
assert.sameValue(result, undefined);

result = arr.findLast(function() {
  return '';
});
assert.sameValue(result, undefined, 'coerced string');

result = arr.findLast(function() {
  return undefined;
});
assert.sameValue(result, undefined, 'coerced undefined');

result = arr.findLast(function() {
  return null;
});
assert.sameValue(result, undefined, 'coerced null');

result = arr.findLast(function() {
  return 0;
});
assert.sameValue(result, undefined, 'coerced 0');

result = arr.findLast(function() {
  return NaN;
});
assert.sameValue(result, undefined, 'coerced NaN');
