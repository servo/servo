// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - callbackfn called with correct parameters
    (initialvalue not passed)
---*/

function callbackfn(prevVal, curVal, idx, obj)
{
  if (idx > 0 && obj[idx] === curVal && obj[idx - 1] === prevVal)
    return curVal;
  else
    return false;
}

var arr = [0, 1, true, null, new Object(), "five"];

assert.sameValue(arr.reduce(callbackfn), "five", 'arr.reduce(callbackfn)');
