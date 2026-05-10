// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map doesn't visit deleted elements when
    Array.length is decreased
---*/

var callCnt = 0;

function callbackfn(val, idx, obj)
{
  srcArr.length = 2;
  callCnt++;
  return 1;
}

var srcArr = [1, 2, 3, 4, 5];
var resArr = srcArr.map(callbackfn);

assert.sameValue(resArr.length, 5, 'resArr.length');
assert.sameValue(callCnt, 2, 'callCnt');
assert.sameValue(resArr[2], undefined, 'resArr[2]');
