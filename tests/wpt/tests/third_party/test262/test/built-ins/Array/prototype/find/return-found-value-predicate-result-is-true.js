// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Return found value if predicate return a boolean true value.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
    e. ReturnIfAbrupt(testResult).
    f. If testResult is true, return kValue.
  ...
features: [Symbol]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var called = 0;

var result = arr.find(function(val) {
  called++;
  return true;
});

assert.sameValue(result, 'Shoes');
assert.sameValue(called, 1, 'predicate was called once');

called = 0;
result = arr.find(function(val) {
  called++;
  return val === 'Bike';
});

assert.sameValue(called, 3, 'predicate was called three times');
assert.sameValue(result, 'Bike');

result = arr.find(function(val) {
  return 'string';
});
assert.sameValue(result, 'Shoes', 'coerced string');

result = arr.find(function(val) {
  return {};
});
assert.sameValue(result, 'Shoes', 'coerced object');

result = arr.find(function(val) {
  return Symbol('');
});
assert.sameValue(result, 'Shoes', 'coerced Symbol');

result = arr.find(function(val) {
  return 1;
});
assert.sameValue(result, 'Shoes', 'coerced number');

result = arr.find(function(val) {
  return -1;
});
assert.sameValue(result, 'Shoes', 'coerced negative number');
