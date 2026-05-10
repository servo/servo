// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The reverse function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.reverse
description: >
    Checking this for Object object, elements are objects and
    primitives, length is string
---*/

var obj = {};
obj.length = "10";
obj.reverse = Array.prototype.reverse;

obj[0] = true;
obj[2] = Infinity;
obj[4] = undefined;
obj[5] = undefined;
obj[8] = "NaN";
obj[9] = "-1";

var reverse = obj.reverse();
if (reverse !== obj) {
  throw new Test262Error('#1: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse() === obj. Actual: ' + (reverse));
}

if (obj[0] !== "-1") {
  throw new Test262Error('#2: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[0] === "-1". Actual: ' + (obj[0]));
}

if (obj[1] !== "NaN") {
  throw new Test262Error('#3: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[1] === "NaN". Actual: ' + (obj[1]));
}

if (obj[2] !== undefined) {
  throw new Test262Error('#4: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[2] === undefined. Actual: ' + (obj[2]));
}

if (obj[3] !== undefined) {
  throw new Test262Error('#5: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[3] === undefined. Actual: ' + (obj[3]));
}

if (obj[4] !== undefined) {
  throw new Test262Error('#6: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[4] === undefined. Actual: ' + (obj[4]));
}

if (obj[5] !== undefined) {
  throw new Test262Error('#7: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[5] === undefined. Actual: ' + (obj[5]));
}

if (obj[6] !== undefined) {
  throw new Test262Error('#8: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[6] === undefined. Actual: ' + (obj[6]));
}

if (obj[7] !== Infinity) {
  throw new Test262Error('#9: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[7] === Infinity. Actual: ' + (obj[7]));
}

if (obj[8] !== undefined) {
  throw new Test262Error('#10: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[8] === undefined. Actual: ' + (obj[8]));
}

if (obj[9] !== true) {
  throw new Test262Error('#11: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj[9] === true. Actual: ' + (obj[9]));
}

obj.length = new String("9");

var reverse = obj.reverse();
if (reverse !== obj) {
  throw new Test262Error('#1: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse() === obj. Actual: ' + (reverse));
}

if (obj[0] !== undefined) {
  throw new Test262Error('#12: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[0] === undefined. Actual: ' + (obj[0]));
}

if (obj[1] !== Infinity) {
  throw new Test262Error('#13: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[1] === Infinity. Actual: ' + (obj[1]));
}

if (obj[2] !== undefined) {
  throw new Test262Error('#14: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[2] === undefined. Actual: ' + (obj[2]));
}

if (obj[3] !== undefined) {
  throw new Test262Error('#15: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[3] === undefined. Actual: ' + (obj[3]));
}

if (obj[4] !== undefined) {
  throw new Test262Error('#16: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[4] === undefined. Actual: ' + (obj[4]));
}

if (obj[5] !== undefined) {
  throw new Test262Error('#17: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[5] === undefined. Actual: ' + (obj[5]));
}

if (obj[6] !== undefined) {
  throw new Test262Error('#18: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[6] === undefined. Actual: ' + (obj[6]));
}

if (obj[7] !== "NaN") {
  throw new Test262Error('#19: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[7] === "NaN". Actual: ' + (obj[7]));
}

if (obj[8] !== "-1") {
  throw new Test262Error('#20: var obj = {}; obj.reverse = Array.prototype.reverse; obj.length = "10"; obj[0] = true; obj[2] = Infinity; obj[4] = undefined; obj[5] = undefined; obj[8] = "NaN"; obj[9] = "-1"; obj.reverse(); obj.length = new String("9"); obj.reverse(); obj[8] === "-1". Actual: ' + (obj[8]));
}
