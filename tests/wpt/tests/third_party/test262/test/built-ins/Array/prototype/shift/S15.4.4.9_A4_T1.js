// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "[[Get]], [[Delete]] from not an inherited property"
esid: sec-array.prototype.shift
description: >
    [[Prototype]] of Array instance is Array.prototype, [[Prototype]
    of Array.prototype is Object.prototype
---*/

Array.prototype[1] = 1;
var x = [0];
x.length = 2;

var shift = x.shift();
if (shift !== 0) {
  throw new Test262Error('#1: Array.prototype[1] = 1; x = [0]; x.length = 2; x.shift() === 0. Actual: ' + (shift));
}

if (x[0] !== 1) {
  throw new Test262Error('#2: Array.prototype[1] = 1; x = [0]; x.length = 2; x.shift(); x[0] === 1. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#3: Array.prototype[1] = 1; x = [0]; x.length = 2; x.shift(); x[1] === 1. Actual: ' + (x[1]));
}

Object.prototype[1] = 1;
Object.prototype.length = 2;
Object.prototype.shift = Array.prototype.shift;
x = {
  0: 0
};

var shift = x.shift();
if (shift !== 0) {
  throw new Test262Error('#4: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.shift = Array.prototype.shift; x = {0:0}; x.shift() === 0. Actual: ' + (shift));
}

if (x[0] !== 1) {
  throw new Test262Error('#5: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.shift = Array.prototype.shift; x = {0:0}; x.shift(); x[0] === 1. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#6: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.shift = Array.prototype.shift; x = {0:0}; x.shift(); x[1] === 1. Actual: ' + (x[1]));
}

if (x.length !== 1) {
  throw new Test262Error('#7: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.shift = Array.prototype.shift; x = {0:0}; x.shift(); x.length === 1. Actual: ' + (x.length));
}

delete x.length;
if (x.length !== 2) {
  throw new Test262Error('#8: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.shift = Array.prototype.shift; x = {0:0}; x.shift(); delete x; x.length === 2. Actual: ' + (x.length));
}
