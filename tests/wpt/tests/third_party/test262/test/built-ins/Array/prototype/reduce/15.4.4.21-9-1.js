// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce doesn't consider new elements added to
    array after it is called
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  arr[5] = 6;
  arr[2] = 3;
  return prevVal + curVal;
}

var arr = [1, 2, , 4, '5'];

assert.sameValue(arr.reduce(callbackfn), "105", 'arr.reduce(callbackfn)');
