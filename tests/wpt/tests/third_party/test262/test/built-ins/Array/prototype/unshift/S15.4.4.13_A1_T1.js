// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The arguments are prepended to the start of the array, such that
    their order within the array is the same as the order in which they appear in
    the argument list
esid: sec-array.prototype.unshift
description: Checking case when unsift is given no arguments or one argument
---*/

var x = new Array();
var unshift = x.unshift(1);
if (unshift !== 1) {
  throw new Test262Error('#1: x = new Array(); x.unshift(1) === 1. Actual: ' + (unshift));
}

if (x[0] !== 1) {
  throw new Test262Error('#2: x = new Array(); x.unshift(1); x[0] === 1. Actual: ' + (x[0]));
}

var unshift = x.unshift();
if (unshift !== 1) {
  throw new Test262Error('#3: x = new Array(); x.unshift(1); x.unshift() === 1. Actual: ' + (unshift));
}

if (x[1] !== undefined) {
  throw new Test262Error('#4: x = new Array(); x.unshift(1); x.unshift(); x[1] === unedfined. Actual: ' + (x[1]));
}

var unshift = x.unshift(-1);
if (unshift !== 2) {
  throw new Test262Error('#5: x = new Array(); x.unshift(1); x.unshift(); x.unshift(-1) === 2. Actual: ' + (unshift));
}

if (x[0] !== -1) {
  throw new Test262Error('#6: x = new Array(); x.unshift(1); x.unshift(-1); x[0] === -1. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#7: x = new Array(); x.unshift(1); x.unshift(-1); x[1] === 1. Actual: ' + (x[1]));
}

if (x.length !== 2) {
  throw new Test262Error('#8: x = new Array(); x.unshift(1); x.unshift(); x.unshift(-1); x.length === 2. Actual: ' + (x.length));
}
