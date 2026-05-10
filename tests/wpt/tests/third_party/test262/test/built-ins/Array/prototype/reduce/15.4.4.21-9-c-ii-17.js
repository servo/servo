// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - 'accumulator' used for current iteration
    is the result of previous iteration on an Array
---*/

var result = true;
var accessed = false;
var preIteration = 1;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  if (preIteration !== prevVal) {
    result = false;
  }
  preIteration = curVal;
  return curVal;
}

[11, 12, 13].reduce(callbackfn, 1);

assert(result, 'result !== true');
assert(accessed, 'accessed !== true');
