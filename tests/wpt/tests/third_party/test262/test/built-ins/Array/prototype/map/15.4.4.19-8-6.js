// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map visits deleted element in array after the call
    when same index is also present in prototype
---*/

function callbackfn(val, idx, obj)
{
  delete srcArr[4];
  if (val > 0)
    return 1;
  else
    return 0;

}

Array.prototype[4] = 5;
var srcArr = [1, 2, 3, 4, 5];
var resArr = srcArr.map(callbackfn);
delete Array.prototype[4];

assert.sameValue(resArr.length, 5, 'resArr.length');
assert.sameValue(resArr[4], 1, 'resArr[4]');
