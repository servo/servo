// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some doesn't visit deleted elements when
    Array.length is decreased
---*/

function callbackfn(val, idx, obj)
{
  arr.length = 3;
  if (val < 4)
    return false;
  else
    return true;
}

var arr = [1, 2, 3, 4, 6];


assert.sameValue(arr.some(callbackfn), false, 'arr.some(callbackfn)');
