// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If start is positive, use min(start, length).
    If end is negative, use max(end + length, 0)
esid: sec-array.prototype.slice
description: length > abs(end) > start > 0, end < 0
---*/

var x = [0, 1, 2, 3, 4];
var arr = x.slice(1, -4);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var x = [0,1,2,3,4]; var arr = x.slice(1,-4); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 0) {
  throw new Test262Error('#2: var x = [0,1,2,3,4]; var arr = x.slice(1,-4); arr.length === 0. Actual: ' + (arr.length));
}

if (arr[0] !== undefined) {
  throw new Test262Error('#3: var x = [0,1,2,3,4]; var arr = x.slice(1,-4); arr[0] === undefined. Actual: ' + (arr[0]));
}
