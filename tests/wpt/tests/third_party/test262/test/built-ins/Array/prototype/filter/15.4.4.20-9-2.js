// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-2
description: >
    Array.prototype.filter considers new value of elements in array
    after it is called
---*/

function callbackfn(val, idx, obj)
{
  srcArr[2] = -1;
  srcArr[4] = -1;
  if (val > 0)
    return true;
  else
    return false;
}

var srcArr = [1, 2, 3, 4, 5];
var resArr = srcArr.filter(callbackfn);

assert.sameValue(resArr.length, 3, 'resArr.length');
assert.sameValue(resArr[0], 1, 'resArr[0]');
assert.sameValue(resArr[2], 4, 'resArr[2]');
