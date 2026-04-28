// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every immediately returns false if callbackfn
    returns false
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  callCnt++;
  if (idx > 5)
    return false;
  else
    return true;
}

var arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];


assert.sameValue(arr.every(callbackfn), false, 'arr.every(callbackfn)');
assert.sameValue(callCnt, 7, 'callCnt');
