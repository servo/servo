// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "[[Get]], [[Delete]] from not an inherited property"
esid: sec-array.prototype.reverse
description: >
    [[Prototype]] of Array instance is Array.prototype, [[Prototype]
    of Array.prototype is Object.prototype
---*/

Array.prototype[1] = 1;
var x = [0];
x.length = 2;
x.reverse();

if (x[0] !== 1) {
  throw new Test262Error('#1: Array.prototype[1] = 1; x = [0]; x.length = 2; x.reverse(); x[0] === 1. Actual: ' + (x[0]));
}

if (x[1] !== 0) {
  throw new Test262Error('#2: Array.prototype[1] = 1; x = [0]; x.length = 2; x.reverse(); x[1] === 0. Actual: ' + (x[1]));
}

x.length = 0;

if (x[0] !== undefined) {
  throw new Test262Error('#3: Array.prototype[1] = 1; x = [0]; x.length = 2; x.reverse(); x.length = 0; x[0] === undefined. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#4: Array.prototype[1] = 1; x = [0]; x.length = 2; x.reverse(); x.length = 0; x[1] === 1. Actual: ' + (x[1]));
}

Object.prototype[1] = 1;
Object.prototype.length = 2;
Object.prototype.reverse = Array.prototype.reverse;
x = {
  0: 0
};
x.reverse();

if (x[0] !== 1) {
  throw new Test262Error('#5: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.reverse = Array.prototype.reverse; x = {0:0}; x.reverse(); x[0] === 0. Actual: ' + (x[0]));
}

if (x[1] !== 0) {
  throw new Test262Error('#6: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.reverse = Array.prototype.reverse; x = {0:0}; x.reverse(); x[1] === 0. Actual: ' + (x[1]));
}

delete x[0];
delete x[1];

if (x[0] !== undefined) {
  throw new Test262Error('#7: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.reverse = Array.prototype.reverse; x = {0:0}; x.reverse(); delete x[0]; delete x[1]; x[0] === undefined. Actual: ' + (x[0]));
}

if (x[1] !== 1) {
  throw new Test262Error('#8: Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.reverse = Array.prototype.reverse; x = {0:0}; x.reverse(); delete x[0]; delete x[1]; x[1] === 1. Actual: ' + (x[1]));
}
