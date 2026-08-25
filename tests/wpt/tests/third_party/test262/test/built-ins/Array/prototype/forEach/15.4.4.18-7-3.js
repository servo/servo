// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach doesn't visit deleted elements when
    Array.length is decreased
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  arr.length = 3;
  callCnt++;
}

var arr = [1, 2, 3, 4, 5];
arr.forEach(callbackfn);

assert.sameValue(callCnt, 3, 'callCnt');
