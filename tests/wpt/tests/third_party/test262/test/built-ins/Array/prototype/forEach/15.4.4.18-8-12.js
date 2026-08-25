// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach doesn't visit expandos
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  callCnt++;
}
var arr = [1, 2, 3, 4, 5];
arr["i"] = 10;
arr[true] = 11;

arr.forEach(callbackfn);

assert.sameValue(callCnt, 5, 'callCnt');
