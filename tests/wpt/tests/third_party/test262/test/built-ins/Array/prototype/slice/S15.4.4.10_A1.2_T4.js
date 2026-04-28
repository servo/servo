// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If start is negative, use max(start + length, 0).
    If end is positive, use min(end, length)
esid: sec-array.prototype.slice
description: abs(start) > length = end > 0, start < 0
---*/

var x = [0, 1, 2, 3, 4];
var arr = x.slice(-9, 5);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 5) {
  throw new Test262Error('#2: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr.length === 5. Actual: ' + (arr.length));
}

if (arr[0] !== 0) {
  throw new Test262Error('#3: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr[0] === 0. Actual: ' + (arr[0]));
}

if (arr[1] !== 1) {
  throw new Test262Error('#4: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr[1] === 1. Actual: ' + (arr[1]));
}

if (arr[2] !== 2) {
  throw new Test262Error('#5: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr[2] === 2. Actual: ' + (arr[2]));
}

if (arr[3] !== 3) {
  throw new Test262Error('#6: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr[3] === 3. Actual: ' + (arr[3]));
}

if (arr[4] !== 4) {
  throw new Test262Error('#7: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr[4] === 4. Actual: ' + (arr[4]));
}

if (arr[5] !== undefined) {
  throw new Test262Error('#8: var x = [0,1,2,3,4]; var arr = x.slice(-9,5); arr[5] === undefined. Actual: ' + (arr[5]));
}
