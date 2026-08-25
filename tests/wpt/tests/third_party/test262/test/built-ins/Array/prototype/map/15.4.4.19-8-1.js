// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map doesn't consider new elements added to array
    after it is called
---*/

function callbackfn(val, idx, obj)
{
  srcArr[2] = 3;
  srcArr[5] = 6;
  return 1;
}

var srcArr = [1, 2, , 4, 5];
var resArr = srcArr.map(callbackfn);

assert.sameValue(resArr.length, 5, 'resArr.length');
