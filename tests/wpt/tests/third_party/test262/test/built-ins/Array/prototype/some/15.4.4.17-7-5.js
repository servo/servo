// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some doesn't consider newly added elements in
    sparse array
---*/

function callbackfn(val, idx, obj)
{
  arr[1000] = 5;
  if (val < 5)
    return false;
  else
    return true;
}

var arr = new Array(10);
arr[1] = 1;
arr[2] = 2;


assert.sameValue(arr.some(callbackfn), false, 'arr.some(callbackfn)');
