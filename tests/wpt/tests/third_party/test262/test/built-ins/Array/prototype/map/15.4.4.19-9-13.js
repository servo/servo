// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - if there are no side effects of the
    functions, O is unmodified
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  return val > 2;
}

var arr = [1, 2, 3, 4];

arr.map(callbackfn);

assert.sameValue(arr[0], 1, 'arr[0]');
assert.sameValue(arr[1], 2, 'arr[1]');
assert.sameValue(arr[2], 3, 'arr[2]');
assert.sameValue(arr[3], 4, 'arr[3]');
assert.sameValue(called, 4, 'called');
