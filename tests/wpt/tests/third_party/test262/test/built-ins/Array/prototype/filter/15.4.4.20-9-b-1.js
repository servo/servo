// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-1
description: >
    Array.prototype.filter - callbackfn not called for indexes never
    been assigned values
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  callCnt++;
  return false;
}

var srcArr = new Array(10);
srcArr[1] = undefined; //explicitly assigning a value
var resArr = srcArr.filter(callbackfn);

assert.sameValue(resArr.length, 0, 'resArr.length');
assert.sameValue(callCnt, 1, 'callCnt');
