// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findindex
description: >
  Return -1 if predicate always returns a boolean false value.
info: |
  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
    ...
  9. Return -1.
features: [Symbol]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var called = 0;

var result = arr.findIndex(function(val) {
  called++;
  return false;
});

assert.sameValue(called, 3, 'predicate was called three times');
assert.sameValue(result, -1);

result = arr.findIndex(function(val) {
  return '';
});
assert.sameValue(result, -1, 'coerced string');

result = arr.findIndex(function(val) {
  return undefined;
});
assert.sameValue(result, -1, 'coerced undefined');

result = arr.findIndex(function(val) {
  return null;
});
assert.sameValue(result, -1, 'coerced null');

result = arr.findIndex(function(val) {
  return 0;
});
assert.sameValue(result, -1, 'coerced 0');

result = arr.findIndex(function(val) {
  return NaN;
});
assert.sameValue(result, -1, 'coerced NaN');
