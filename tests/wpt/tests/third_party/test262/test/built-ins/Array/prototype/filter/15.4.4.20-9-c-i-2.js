// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - element to be retrieved is own data
    property on an Array
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 11;
  }
}

var newArr = [11].filter(callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
