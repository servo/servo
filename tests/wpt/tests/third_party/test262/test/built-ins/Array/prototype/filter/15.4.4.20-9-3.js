// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-3
description: >
    Array.prototype.filter doesn't visit deleted elements in array
    after the call
---*/

function callbackfn(val, idx, obj)
{
  delete srcArr[2];
  delete srcArr[4];
  if (val > 0)
    return true;
  else
    return false;
}

var srcArr = [1, 2, 3, 4, 5];
var resArr = srcArr.filter(callbackfn);

// two elements deleted
assert.sameValue(resArr.length, 3, 'resArr.length');
assert.sameValue(resArr[0], 1, 'resArr[0]');
assert.sameValue(resArr[2], 4, 'resArr[2]');
