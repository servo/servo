// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The slice function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.slice
description: >
    If start is negative, use max(start + length, 0).  If end is
    positive, use min(end, length)
---*/

var obj = {};
obj.slice = Array.prototype.slice;
obj[0] = 0;
obj[1] = 1;
obj[2] = 2;
obj[3] = 3;
obj[4] = 4;
obj.length = 5;
var arr = obj.slice(-5, 3);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(-5,3); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 3) {
  throw new Test262Error('#2: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(-5,3); arr.length === 3. Actual: ' + (arr.length));
}

if (arr[0] !== 0) {
  throw new Test262Error('#3: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(-5,3); arr[0] === 0. Actual: ' + (arr[0]));
}

if (arr[1] !== 1) {
  throw new Test262Error('#4: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(-5,3); arr[1] === 1. Actual: ' + (arr[1]));
}

if (arr[2] !== 2) {
  throw new Test262Error('#5: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(-5,3); arr[2] === 2. Actual: ' + (arr[2]));
}

if (arr[3] !== undefined) {
  throw new Test262Error('#6: var obj = {}; obj.slice = Array.prototype.slice; obj[0] = 0; obj[1] = 1; obj[2] = 2; obj[3] = 3; obj[4] = 4; obj.length = 5; var arr = obj.slice(-5,3); arr[3] === undefined. Actual: ' + (arr[3]));
}
