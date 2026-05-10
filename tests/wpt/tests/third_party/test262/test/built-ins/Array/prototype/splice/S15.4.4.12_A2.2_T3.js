// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToInteger from deleteCount
esid: sec-array.prototype.splice
description: deleteCount = Infinity
---*/

var x = [0, 1, 2, 3];
var arr = x.splice(0, Number.POSITIVE_INFINITY);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var x = [0,1,2,3]; var arr = x.splice(0,Number.POSITIVE_INFINITY); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 4) {
  throw new Test262Error('#2: var x = [0,1,2,3]; var arr = x.splice(0,Number.POSITIVE_INFINITY); arr.length === 4. Actual: ' + (arr.length));
}

if (arr[0] !== 0) {
  throw new Test262Error('#3: var x = [0,1,2,3]; var arr = x.splice(0,Number.POSITIVE_INFINITY); arr[0] === 0. Actual: ' + (arr[0]));
}

if (arr[1] !== 1) {
  throw new Test262Error('#4: var x = [0,1,2,3]; var arr = x.splice(0,Number.POSITIVE_INFINITY); arr[1] === 1. Actual: ' + (arr[1]));
}

if (arr[2] !== 2) {
  throw new Test262Error('#5: var x = [0,1,2,3]; var arr = x.splice(0,Number.POSITIVE_INFINITY); arr[2] === 2. Actual: ' + (arr[2]));
}

if (arr[3] !== 3) {
  throw new Test262Error('#6: var x = [0,1,2,3]; var arr = x.splice(0,Number.POSITIVE_INFINITY); arr[3] === 3. Actual: ' + (arr[3]));
}

if (x.length !== 0) {
  throw new Test262Error('#7: var x = [0,1,2,3]; var arr = x.splice(0,Number.POSITIVE_INFINITY); x.length === 0. Actual: ' + (x.length));
}
