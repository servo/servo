// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlast
description: >
  Return found value if predicate return a boolean true value.
info: |
  Array.prototype.findLast ( predicate[ , thisArg ] )

  ...
  5. Repeat, while k ≥ 0,
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    d. If testResult is true, return kValue.
  ...
features: [Symbol, array-find-from-last]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var called = 0;

var result = arr.findLast(function() {
  called++;
  return true;
});

assert.sameValue(result, 'Bike');
assert.sameValue(called, 1, 'predicate was called once');

called = 0;
result = arr.findLast(function(val) {
  called++;
  return val === 'Shoes';
});

assert.sameValue(called, 3, 'predicate was called three times');
assert.sameValue(result, 'Shoes');

result = arr.findLast(function() {
  return 'string';
});
assert.sameValue(result, 'Bike', 'coerced string');

result = arr.findLast(function() {
  return {};
});
assert.sameValue(result, 'Bike', 'coerced object');

result = arr.findLast(function() {
  return Symbol('');
});
assert.sameValue(result, 'Bike', 'coerced Symbol');

result = arr.findLast(function() {
  return 1;
});
assert.sameValue(result, 'Bike', 'coerced number');

result = arr.findLast(function() {
  return -1;
});
assert.sameValue(result, 'Bike', 'coerced negative number');
