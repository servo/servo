// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every doesn't visit deleted elements when
    Array.length is decreased
---*/

function callbackfn(val, Idx, obj)
{
  arr.length = 3;
  if (val < 4)
    return true;
  else
    return false;
}

var arr = [1, 2, 3, 4, 6];


assert.sameValue(arr.every(callbackfn), true, 'arr.every(callbackfn)');
