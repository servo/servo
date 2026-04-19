// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - 'callbackfn' is a function
---*/

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    return val === 9;
  }
  return false;
}

var newArr = [11, 9].filter(callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 9, 'newArr[0]');
