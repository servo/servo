// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - callbackfn called with correct parameters
---*/

var bPar = true;
var bCalled = false;

function callbackfn(val, idx, obj)
{
  bCalled = true;
  if (obj[idx] !== val)
    bPar = false;
}

var srcArr = [0, 1, true, null, new Object(), "five"];
srcArr[999999] = -6.6;
var resArr = srcArr.map(callbackfn);


assert.sameValue(bCalled, true, 'bCalled');
assert.sameValue(bPar, true, 'bPar');
