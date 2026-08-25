// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - callbackfn not called for indexes never been
    assigned values
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  callCnt++;
  return 1;
}

var srcArr = new Array(10);
srcArr[1] = undefined; //explicitly assigning a value
var resArr = srcArr.map(callbackfn);

assert.sameValue(resArr.length, 10, 'resArr.length');
assert.sameValue(callCnt, 1, 'callCnt');
