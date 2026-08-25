// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlastindex
description: >
  The range of elements processed is set before the first call to `predicate`.
info: |
  Array.prototype.findLastIndex ( predicate[ , thisArg ] )

  ...
  4. Let k be len - 1.
  5. Repeat, while k ≥ 0,
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...
features: [array-find-from-last]
---*/

var arr = ['Shoes', 'Car', 'Bike'];
var results = [];

arr.findLastIndex(function(kValue) {
  if (results.length === 0) {
    arr.splice(1, 1);
  }
  results.push(kValue);
});

assert.sameValue(results.length, 3, 'predicate called three times');
assert.sameValue(results[0], 'Bike');
assert.sameValue(results[1], 'Bike');
assert.sameValue(results[2], 'Shoes');

results = [];
arr = ['Skateboard', 'Barefoot'];
arr.findLastIndex(function(kValue) {
  if (results.length === 0) {
    arr.push('Motorcycle');
    arr[0] = 'Magic Carpet';
  }

  results.push(kValue);
});

assert.sameValue(results.length, 2, 'predicate called twice');
assert.sameValue(results[0], 'Barefoot');
assert.sameValue(results[1], 'Magic Carpet');
