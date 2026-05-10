// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If start is negative, use max(start + length, 0).
    If end is positive, use min(end, length)
esid: sec-array.prototype.slice
description: length = end > abs(start), start < 0
---*/

var x = [0, 1, 2, 3, 4];
var arr = x.slice(-1, 5);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var x = [0,1,2,3,4]; var arr = x.slice(-1,5); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 1) {
  throw new Test262Error('#2: var x = [0,1,2,3,4]; var arr = x.slice(-1,5); arr.length === 1. Actual: ' + (arr.length));
}

if (arr[0] !== 4) {
  throw new Test262Error('#3: var x = [0,1,2,3,4]; var arr = x.slice(-1,5); arr[0] === 4. Actual: ' + (arr[0]));
}

if (arr[1] !== undefined) {
  throw new Test262Error('#4: var x = [0,1,2,3,4]; var arr = x.slice(-1,5); arr[1] === undefined. Actual: ' + (arr[1]));
}
