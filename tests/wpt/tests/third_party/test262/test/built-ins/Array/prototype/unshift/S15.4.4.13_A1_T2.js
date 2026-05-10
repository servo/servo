// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The arguments are prepended to the start of the array, such that
    their order within the array is the same as the order in which they appear in
    the argument list
esid: sec-array.prototype.unshift
description: Checking case when unsift is given many arguments
---*/

var x = [];
if (x.length !== 0) {
  throw new Test262Error('#1: x = []; x.length === 0. Actual: ' + (x.length));
}

x[0] = 0;
var unshift = x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1);
if (unshift !== 6) {
  throw new Test262Error('#2: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1) === 6. Actual: ' + (unshift));
}

if (x[5] !== 0) {
  throw new Test262Error('#3: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[5] === 0. Actual: ' + (x[5]));
}

if (x[0] !== true) {
  throw new Test262Error('#4: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[0] === true. Actual: ' + (x[0]));
}

if (x[1] !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#5: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[1] === Number.POSITIVE_INFINITY. Actual: ' + (x[1]));
}

if (x[2] !== "NaN") {
  throw new Test262Error('#6: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[2] === "NaN". Actual: ' + (x[2]));
}

if (x[3] !== "1") {
  throw new Test262Error('#7: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[3] === "1". Actual: ' + (x[3]));
}

if (x[4] !== -1) {
  throw new Test262Error('#8: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x[4] === -1. Actual: ' + (x[4]));
}

if (x.length !== 6) {
  throw new Test262Error('#9: x = []; x[0] = 0; x.unshift(true, Number.POSITIVE_INFINITY, "NaN", "1", -1); x.length === 6. Actual: ' + (x.length));
}
