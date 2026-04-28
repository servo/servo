// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToInteger from start
esid: sec-array.prototype.splice
description: start = Infinity
---*/

var x = [0, 1, 2, 3];
var arr = x.splice(Number.POSITIVE_INFINITY, 3);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#1: var x = [0,1,2,3]; var arr = x.splice(Number.POSITIVE_INFINITY,3); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 0) {
  throw new Test262Error('#2: var x = [0,1,2,3]; var arr = x.splice(Number.POSITIVE_INFINITY,3); arr.length === 0. Actual: ' + (arr.length));
}

if (x[0] !== 0) {
  throw new Test262Error('#3: var x = [0,1,2,3]; var x = x.splice(Number.POSITIVE_INFINITY,3); x[0] === 0. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#4: var x = [0,1,2,3]; var x = x.splice(Number.POSITIVE_INFINITY,3); x[1] === 1. Actual: ' + (x[1]));
}

if (x[2] !== 2) {
  throw new Test262Error('#5: var x = [0,1,2,3]; var x = x.splice(Number.POSITIVE_INFINITY,3); x[2] === 2. Actual: ' + (x[2]));
}

if (x[3] !== 3) {
  throw new Test262Error('#6: var x = [0,1,2,3]; var x = x.splice(Number.POSITIVE_INFINITY,3); x[3] === 3. Actual: ' + (x[3]));
}
