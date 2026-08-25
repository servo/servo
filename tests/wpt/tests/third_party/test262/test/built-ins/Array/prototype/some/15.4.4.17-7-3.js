// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some doesn't visit deleted elements in array after
    it is called
---*/

function callbackfn(val, idx, obj)
{
  delete arr[2];
  if (val !== 3)
    return false;
  else
    return true;
}

var arr = [1, 2, 3, 4, 5];


assert.sameValue(arr.some(callbackfn), false, 'arr.some(callbackfn)');
