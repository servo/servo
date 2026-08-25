// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
    Array.prototype.splice - 'from' is the result of
    ToString(actualStart+k) in an Array
---*/

var arrObj = [1, 2, 3];
var newArrObj = arrObj.splice(-2, 1);

assert.sameValue(newArrObj.length, 1, 'newArrObj.length');
assert.sameValue(newArrObj[0], 2, 'newArrObj[0]');
