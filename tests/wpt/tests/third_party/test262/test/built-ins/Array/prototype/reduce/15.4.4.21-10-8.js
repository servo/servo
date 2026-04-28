// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce doesn't visit expandos
---*/

var callCnt = 0;

function callbackfn(prevVal, curVal, idx, obj)
{
  callCnt++;
  return curVal;
}
var srcArr = ['1', '2', '3', '4', '5'];
srcArr["i"] = 10;
srcArr[true] = 11;
srcArr.reduce(callbackfn);

assert.sameValue(callCnt, 4, 'callCnt');
