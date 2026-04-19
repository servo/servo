// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - callbackfn called with correct parameters
---*/

function callbackfn(val, idx, obj)
{
  if (obj[idx] === val)
    return false;
  else
    return true;
}

var arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];


assert.sameValue(arr.some(callbackfn), false, 'arr.some(callbackfn)');
