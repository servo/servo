// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - callbackfn called with correct parameters
---*/

var bPar = true;
var bCalled = false;

function callbackfn(val, idx, obj)
{
  bCalled = true;
  if (obj[idx] !== val)
    bPar = false;
}

var arr = [0, 1, true, null, new Object(), "five"];
arr[999999] = -6.6;
arr.forEach(callbackfn);

assert.sameValue(bCalled, true, 'bCalled');
assert.sameValue(bPar, true, 'bPar');
