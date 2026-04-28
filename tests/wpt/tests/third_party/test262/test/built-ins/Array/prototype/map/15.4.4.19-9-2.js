// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map returns new Array with same number of elements
    and values the result of callbackfn
---*/

function callbackfn(val, idx, obj)
{
  return val + 10;
}
var srcArr = [1, 2, 3, 4, 5];
var resArr = srcArr.map(callbackfn);

assert.sameValue(resArr[0], 11, 'resArr[0]');
assert.sameValue(resArr[1], 12, 'resArr[1]');
assert.sameValue(resArr[2], 13, 'resArr[2]');
assert.sameValue(resArr[3], 14, 'resArr[3]');
assert.sameValue(resArr[4], 15, 'resArr[4]');
