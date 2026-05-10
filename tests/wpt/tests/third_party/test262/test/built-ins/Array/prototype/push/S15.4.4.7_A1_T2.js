// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The arguments are appended to the end of the array, in
    the order in which they appear. The new length of the array is returned
    as the result of the call
esid: sec-array.prototype.push
description: Checking case when push is given many arguments
---*/

var x = [];
if (x.length !== 0) {
  throw new Test262Error('#1: x = []; x.length === 0. Actual: ' + (x.length));
}

x[0] = 0;
var push = x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1);
if (push !== 6) {
  throw new Test262Error('#2: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1) === 6. Actual: ' + (push));
}

if (x[0] !== 0) {
  throw new Test262Error('#3: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[0] === 0. Actual: ' + (x[0]));
}

if (x[1] !== true) {
  throw new Test262Error('#4: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[1] === true. Actual: ' + (x[1]));
}

if (x[2] !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#5: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[2] === Number.POSITIVE_INFINITY. Actual: ' + (x[2]));
}

if (x[3] !== "NaN") {
  throw new Test262Error('#6: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[3] === "NaN". Actual: ' + (x[3]));
}

if (x[4] !== "1") {
  throw new Test262Error('#7: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[4] === "1". Actual: ' + (x[4]));
}

if (x[5] !== -1) {
  throw new Test262Error('#8: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[5] === -1. Actual: ' + (x[5]));
}

if (x.length !== 6) {
  throw new Test262Error('#9: x = []; x[0] = 0; x.push(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x.length === 6. Actual: ' + (x.length));
}
