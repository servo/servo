// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The elements of the array are rearranged so as to reverse their order.
    The object is returned as the result of the call
esid: sec-array.prototype.reverse
description: Checking this algorithm, elements are objects and primitives
---*/

var x = [];
x[0] = true;
x[2] = Infinity;
x[4] = undefined;
x[5] = undefined;
x[8] = "NaN";
x[9] = "-1";

var reverse = x.reverse();
if (reverse !== x) {
  throw new Test262Error('#1: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse() === x. Actual: ' + (reverse));
}

if (x[0] !== "-1") {
  throw new Test262Error('#2: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[0] === "-1". Actual: ' + (x[0]));
}

if (x[1] !== "NaN") {
  throw new Test262Error('#3: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[1] === "NaN". Actual: ' + (x[1]));
}

if (x[2] !== undefined) {
  throw new Test262Error('#4: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[2] === undefined. Actual: ' + (x[2]));
}

if (x[3] !== undefined) {
  throw new Test262Error('#5: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[3] === undefined. Actual: ' + (x[3]));
}

if (x[4] !== undefined) {
  throw new Test262Error('#6: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[4] === undefined. Actual: ' + (x[4]));
}

if (x[5] !== undefined) {
  throw new Test262Error('#7: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[5] === undefined. Actual: ' + (x[5]));
}

if (x[6] !== undefined) {
  throw new Test262Error('#8: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[6] === undefined. Actual: ' + (x[6]));
}

if (x[7] !== Infinity) {
  throw new Test262Error('#9: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[7] === Infinity. Actual: ' + (x[7]));
}

if (x[8] !== undefined) {
  throw new Test262Error('#10: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[8] === undefined. Actual: ' + (x[8]));
}

if (x[9] !== true) {
  throw new Test262Error('#11: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x[9] === true. Actual: ' + (x[9]));
}

x.length = 9;

var reverse = x.reverse();
if (reverse !== x) {
  throw new Test262Error('#1: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse() === x. Actual: ' + (reverse));
}

if (x[0] !== undefined) {
  throw new Test262Error('#12: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[0] === undefined. Actual: ' + (x[0]));
}

if (x[1] !== Infinity) {
  throw new Test262Error('#13: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[1] === Infinity. Actual: ' + (x[1]));
}

if (x[2] !== undefined) {
  throw new Test262Error('#14: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[2] === undefined. Actual: ' + (x[2]));
}

if (x[3] !== undefined) {
  throw new Test262Error('#15: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[3] === undefined. Actual: ' + (x[3]));
}

if (x[4] !== undefined) {
  throw new Test262Error('#16: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[4] === undefined. Actual: ' + (x[4]));
}

if (x[5] !== undefined) {
  throw new Test262Error('#17: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[5] === undefined. Actual: ' + (x[5]));
}

if (x[6] !== undefined) {
  throw new Test262Error('#18: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[6] === undefined. Actual: ' + (x[6]));
}

if (x[7] !== "NaN") {
  throw new Test262Error('#19: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[7] === "NaN". Actual: ' + (x[7]));
}

if (x[8] !== "-1") {
  throw new Test262Error('#20: x = []; x[0] = true; x[2] = Infinity; x[4] = undefined; x[5] = undefined; x[8] = "NaN"; x[9] = "-1"; x.reverse(); x.length = 9; x.reverse(); x[8] === "-1". Actual: ' + (x[8]));
}
