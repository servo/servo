// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach doesn't consider new elements added to
    array after the call
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  callCnt++;
  arr[2] = 3;
  arr[5] = 6;
}

var arr = [1, 2, , 4, 5];
arr.forEach(callbackfn);

assert.sameValue(callCnt, 5, 'callCnt');
