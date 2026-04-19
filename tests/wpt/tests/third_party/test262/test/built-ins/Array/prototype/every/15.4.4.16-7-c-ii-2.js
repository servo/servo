// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - callbackfn takes 3 arguments
---*/

function callbackfn(val, Idx, obj)
{
  if (arguments.length === 3) //verify if callbackfn was called with 3 parameters
    return true;
}

var arr = [0, 1, true, null, new Object(), "five"];
arr[999999] = -6.6;


assert.sameValue(arr.every(callbackfn), true, 'arr.every(callbackfn)');
