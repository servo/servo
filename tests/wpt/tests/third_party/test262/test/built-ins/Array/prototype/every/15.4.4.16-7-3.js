// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every doesn't visit deleted elements in array
    after the call
---*/

function callbackfn(val, Idx, obj)
{
  delete arr[2];
  if (val == 3)
    return false;
  else
    return true;
}

var arr = [1, 2, 3, 4, 5];


assert.sameValue(arr.every(callbackfn), true, 'arr.every(callbackfn)');
