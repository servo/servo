// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach doesn't consider newly added elements in
    sparse array
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  arr[1000] = 3;
  callCnt++;
}

var arr = new Array(10);
arr[1] = 1;
arr[2] = 2;
arr.forEach(callbackfn);

assert.sameValue(callCnt, 2, 'callCnt');
