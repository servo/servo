// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToInteger from start
esid: sec-array.prototype.slice
description: start is not integer
---*/

var x = [0, 1, 2, 3, 4];
var arr = x.slice(2.5, 4);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var x = [0,1,2,3,4]; var arr = x.slice(2.5,4); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 2) {
  throw new Test262Error('#2: var x = [0,1,2,3,4]; var arr = x.slice(2.5,4); arr.length === 2. Actual: ' + (arr.length));
}

if (arr[0] !== 2) {
  throw new Test262Error('#3: var x = [0,1,2,3,4]; var arr = x.slice(2.5,4); arr[0] === 2. Actual: ' + (arr[0]));
}

if (arr[1] !== 3) {
  throw new Test262Error('#4: var x = [0,1,2,3,4]; var arr = x.slice(2.5,4); arr[1] === 3. Actual: ' + (arr[1]));
}

if (arr[3] !== undefined) {
  throw new Test262Error('#5: var x = [0,1,2,3,4]; var arr = x.slice(2.5,4); arr[3] === undefined. Actual: ' + (arr[3]));
}
