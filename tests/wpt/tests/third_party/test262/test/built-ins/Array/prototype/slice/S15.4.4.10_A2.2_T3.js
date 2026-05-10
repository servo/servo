// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToInteger from end
esid: sec-array.prototype.slice
description: end = Infinity
---*/

var x = [0, 1, 2, 3, 4];
var arr = x.slice(0, Number.POSITIVE_INFINITY);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 5) {
  throw new Test262Error('#2: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr.length === 5. Actual: ' + (arr.length));
}

if (arr[0] !== 0) {
  throw new Test262Error('#3: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr[0] === 0. Actual: ' + (arr[0]));
}

if (arr[1] !== 1) {
  throw new Test262Error('#4: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr[1] === 1. Actual: ' + (arr[1]));
}

if (arr[2] !== 2) {
  throw new Test262Error('#5: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr[2] === 2. Actual: ' + (arr[2]));
}

if (arr[3] !== 3) {
  throw new Test262Error('#6: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr[3] === 3. Actual: ' + (arr[3]));
}

if (arr[4] !== 4) {
  throw new Test262Error('#7: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr[4] === 4. Actual: ' + (arr[4]));
}

if (arr[5] !== undefined) {
  throw new Test262Error('#8: var x = [0,1,2,3,4]; var arr = x.slice(0,Number.POSITIVE_INFINITY); arr[5] === undefined. Actual: ' + (arr[5]));
}
