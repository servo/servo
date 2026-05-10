// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce doesn't mutate the Array on which it is
    called on
---*/

function callbackfn(prevVal, curVal, idx, obj)
{
  return 1;
}
var srcArr = [1, 2, 3, 4, 5];
srcArr.reduce(callbackfn);

assert.sameValue(srcArr[0], 1, 'srcArr[0]');
assert.sameValue(srcArr[1], 2, 'srcArr[1]');
assert.sameValue(srcArr[2], 3, 'srcArr[2]');
assert.sameValue(srcArr[3], 4, 'srcArr[3]');
assert.sameValue(srcArr[4], 5, 'srcArr[4]');
