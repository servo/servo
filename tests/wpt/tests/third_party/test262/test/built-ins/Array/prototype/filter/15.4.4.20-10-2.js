// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter returns new Array with length equal to
    number of true returned by callbackfn
---*/

function callbackfn(val, idx, obj)
{
  if (val % 2)
    return true;
  else
    return false;
}
var srcArr = [1, 2, 3, 4, 5];
var resArr = srcArr.filter(callbackfn);

assert.sameValue(resArr.length, 3, 'resArr.length');
assert.sameValue(resArr[0], 1, 'resArr[0]');
assert.sameValue(resArr[1], 3, 'resArr[1]');
assert.sameValue(resArr[2], 5, 'resArr[2]');
