// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - element to be retrieved is own data
    property that overrides an inherited data property on an Array
---*/

function callbackfn(val, idx, obj) {
  return (idx === 0) && (val === 12);
}

Array.prototype[0] = 11;
var newArr = [12].filter(callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 12, 'newArr[0]');
