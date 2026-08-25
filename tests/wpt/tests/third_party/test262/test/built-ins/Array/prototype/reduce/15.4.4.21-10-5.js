// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce reduces the array in ascending order of
    indices(initialvalue present)
---*/

function callbackfn(prevVal, curVal, idx, obj)
{
  return prevVal + curVal;
}
var srcArr = ['1', '2', '3', '4', '5'];

assert.sameValue(srcArr.reduce(callbackfn, '0'), '012345', 'srcArr.reduce(callbackfn,"0")');
