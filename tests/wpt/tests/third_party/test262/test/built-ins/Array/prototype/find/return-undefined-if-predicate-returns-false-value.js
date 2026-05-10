// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Return undefined if predicate always returns a boolean false value.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
    ...
  9. Return undefined.
features: [Symbol]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var called = 0;

var result = arr.find(function(val) {
  called++;
  return false;
});

assert.sameValue(called, 3, 'predicate was called three times');
assert.sameValue(result, undefined);

result = arr.find(function(val) {
  return '';
});
assert.sameValue(result, undefined, 'coerced string');

result = arr.find(function(val) {
  return undefined;
});
assert.sameValue(result, undefined, 'coerced undefined');

result = arr.find(function(val) {
  return null;
});
assert.sameValue(result, undefined, 'coerced null');

result = arr.find(function(val) {
  return 0;
});
assert.sameValue(result, undefined, 'coerced 0');

result = arr.find(function(val) {
  return NaN;
});
assert.sameValue(result, undefined, 'coerced NaN');
