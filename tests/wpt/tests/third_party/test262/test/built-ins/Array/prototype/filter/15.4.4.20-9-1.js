// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-1
description: >
    Array.prototype.filter doesn't consider new elements added to
    array after it is called
---*/

function callbackfn(val, idx, obj) {
  srcArr[2] = 3;
  srcArr[5] = 6;
  return true;
}

var srcArr = [1, 2, , 4, 5];
var resArr = srcArr.filter(callbackfn);

assert.sameValue(resArr.length, 5, 'resArr.length');
