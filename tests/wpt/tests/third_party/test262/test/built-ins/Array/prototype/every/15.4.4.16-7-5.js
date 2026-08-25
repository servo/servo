// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every doesn't consider newly added elements in
    sparse array
---*/

function callbackfn(val, Idx, obj)
{
  arr[1000] = 3;
  if (val < 3)
    return true;
  else
    return false;
}

var arr = new Array(10);
arr[1] = 1;
arr[2] = 2;


assert.sameValue(arr.every(callbackfn), true, 'arr.every(callbackfn)');
