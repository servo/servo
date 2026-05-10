// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findindex
description: >
  Return index if predicate return a boolean true value.
info: |
  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
    e. ReturnIfAbrupt(testResult).
    f. If testResult is true, return k.
  ...
features: [Symbol]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var called = 0;

var result = arr.findIndex(function(val) {
  called++;
  return true;
});

assert.sameValue(result, 0);
assert.sameValue(called, 1, 'predicate was called once');

called = 0;
result = arr.findIndex(function(val) {
  called++;
  return val === 'Bike';
});

assert.sameValue(called, 3, 'predicate was called three times');
assert.sameValue(result, 2);

result = arr.findIndex(function(val) {
  return 'string';
});
assert.sameValue(result, 0, 'coerced string');

result = arr.findIndex(function(val) {
  return {};
});
assert.sameValue(result, 0, 'coerced object');

result = arr.findIndex(function(val) {
  return Symbol('');
});
assert.sameValue(result, 0, 'coerced Symbol');

result = arr.findIndex(function(val) {
  return 1;
});
assert.sameValue(result, 0, 'coerced number');

result = arr.findIndex(function(val) {
  return -1;
});
assert.sameValue(result, 0, 'coerced negative number');
