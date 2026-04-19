// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "[[Get]] from not an inherited property"
esid: sec-array.prototype.push
description: >
    [[Prototype]] of Array instance is Array.prototype, [[Prototype]
    of Array.prototype is Object.prototype
---*/

Object.prototype[1] = -1;
Object.prototype.length = 1;
Object.prototype.push = Array.prototype.push;
var x = {
  0: 0
};

var push = x.push(1);
if (push !== 2) {
  throw new Test262Error('#1: Object.prototype[1] = 1; Object.prototype.length = -1; Object.prototype.push = Array.prototype.push; x = {0:0}; x.push(1) === 2. Actual: ' + (push));
}

if (x.length !== 2) {
  throw new Test262Error('#2: Object.prototype[1] = 1; Object.prototype.length = -1; Object.prototype.push = Array.prototype.push; x = {0:0}; x.push(1); x.length === 2. Actual: ' + (x.length));
}

if (x[1] !== 1) {
  throw new Test262Error('#3: Object.prototype[1] = 1; Object.prototype.length = -1; Object.prototype.push = Array.prototype.push; x = {0:0}; x.push(1); x[1] === 1. Actual: ' + (x[1]));
}

delete x[1];
if (x[1] !== -1) {
  throw new Test262Error('#4: Object.prototype[1] = 1; Object.prototype.length = -1; Object.prototype.push = Array.prototype.push; x = {0:0}; x.push(1); delete x[1]; x[1] === -1. Actual: ' + (x[1]));
}

delete x.length;
if (x.length !== 1) {
  throw new Test262Error('#5: Object.prototype[1] = 1; Object.prototype.length = -1; Object.prototype.push = Array.prototype.push; x = {0:0}; delete x; x.push(1); x.length === 1. Actual: ' + (x.length));
}
