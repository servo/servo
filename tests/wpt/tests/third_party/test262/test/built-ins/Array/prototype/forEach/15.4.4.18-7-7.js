// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - considers new value of elements in array
    after the call
---*/

var result = false;
var arr = [1, 2, 3, 4, 5];

function callbackfn(val, Idx, obj) {
  arr[4] = 6;
  if (val >= 6) {
    result = true;
  }
}

arr.forEach(callbackfn);

assert(result, 'result !== true');
