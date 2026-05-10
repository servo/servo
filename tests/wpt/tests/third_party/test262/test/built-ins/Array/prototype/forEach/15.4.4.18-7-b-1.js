// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - callbackfn not called for indexes never
    been assigned values
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  callCnt++;
}

var arr = new Array(10);
arr[1] = undefined;
arr.forEach(callbackfn);

assert.sameValue(callCnt, 1, 'callCnt');
