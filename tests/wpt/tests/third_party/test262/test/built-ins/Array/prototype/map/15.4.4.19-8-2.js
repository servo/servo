// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map considers new value of elements in array after
    it is called
---*/

function callbackfn(val, idx, obj)
{
  srcArr[4] = -1;
  if (val > 0)
    return 1;
  else
    return 0;
}

var srcArr = [1, 2, 3, 4, 5];
var resArr = srcArr.map(callbackfn);

assert.sameValue(resArr.length, 5, 'resArr.length');
assert.sameValue(resArr[4], 0, 'resArr[4]');
