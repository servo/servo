// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  The range of elements processed is set before the first call to `predicate`.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  6. If thisArg was supplied, let T be thisArg; else let T be undefined.
  7. Let k be 0.
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
  ...
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var results = [];

arr.find(function(kValue) {
  if (results.length === 0) {
    arr.splice(1, 1);
  }
  results.push(kValue);
});

assert.sameValue(results.length, 3, 'predicate called three times');
assert.sameValue(results[0], 'Shoes');
assert.sameValue(results[1], 'Bike');
assert.sameValue(results[2], undefined);

results = [];
arr = ['Skateboard', 'Barefoot'];
arr.find(function(kValue) {
  if (results.length === 0) {
    arr.push('Motorcycle');
    arr[1] = 'Magic Carpet';
  }

  results.push(kValue);
});

assert.sameValue(results.length, 2, 'predicate called twice');
assert.sameValue(results[0], 'Skateboard');
assert.sameValue(results[1], 'Magic Carpet');
