// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce throws TypeError when elements assigned
    values are deleted and initialValue is not present
---*/

function callbackfn(prevVal, curVal, idx, obj)
{}

var arr = [1, 2, 3, 4, 5];
delete arr[0];
delete arr[1];
delete arr[2];
delete arr[3];
delete arr[4];
assert.throws(TypeError, function() {
  arr.reduce(callbackfn);
});
