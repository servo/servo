// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - k values are passed in acending numeric
    order on an Array
---*/

var arr = [0, 1, 2];
var lastIdx = 0;
var result = true;
var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  if (lastIdx !== idx) {
    result = false;
  } else {
    lastIdx++;
  }
}

arr.reduce(callbackfn, 11);

assert(result, 'result !== true');
assert(accessed, 'accessed !== true');
