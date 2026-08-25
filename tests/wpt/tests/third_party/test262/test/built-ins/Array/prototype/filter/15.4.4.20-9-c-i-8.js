// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - element to be retrieved is inherited data
    property on an Array
---*/

function callbackfn(val, idx, obj) {
  return (idx === 1) && (val === 13);
}

Array.prototype[1] = 13;
var newArr = [, , , ].filter(callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 13, 'newArr[0]');
