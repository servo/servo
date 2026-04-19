// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The push function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.push
description: >
    The arguments are appended to the end of the array, in  the order
    in which they appear. The new length of the array is returned  as
    the result of the call
---*/

var obj = {};
obj.push = Array.prototype.push;

if (obj.length !== undefined) {
  throw new Test262Error('#0: var obj = {}; obj.length === undefined. Actual: ' + (obj.length));
} else {
  var push = obj.push(-1);
  if (push !== 1) {
    throw new Test262Error('#1: var obj = {}; obj.push = Array.prototype.push; obj.push(-1) === 1. Actual: ' + (push));
  }
  if (obj.length !== 1) {
    throw new Test262Error('#2: var obj = {}; obj.push = Array.prototype.push; obj.push(-1); obj.length === 1. Actual: ' + (obj.length));
  }
  if (obj["0"] !== -1) {
    throw new Test262Error('#3: var obj = {}; obj.push = Array.prototype.push; obj.push(-1); obj["0"] === -1. Actual: ' + (obj["0"]));
  }
}

obj.length = undefined;
var push = obj.push(-4);
if (push !== 1) {
  throw new Test262Error('#4: var obj = {}; obj.length = undefined; obj.push = Array.prototype.push; obj.push(-4) === 1. Actual: ' + (push));
}

if (obj.length !== 1) {
  throw new Test262Error('#5: var obj = {}; obj.length = undefined; obj.push = Array.prototype.push; obj.push(-4); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -4) {
  throw new Test262Error('#6: var obj = {}; obj.length = undefined; obj.push = Array.prototype.push; obj.push(-4); obj["0"] === -4. Actual: ' + (obj["0"]));
}

obj.length = null
var push = obj.push(-7);
if (push !== 1) {
  throw new Test262Error('#7: var obj = {}; obj.length = null; obj.push = Array.prototype.push; obj.push(-7) === 1. Actual: ' + (push));
}

if (obj.length !== 1) {
  throw new Test262Error('#8: var obj = {}; obj.length = null; obj.push = Array.prototype.push; obj.push(-7); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -7) {
  throw new Test262Error('#9: var obj = {}; obj.length = null; obj.push = Array.prototype.push; obj.push(-7); obj["0"] === -7. Actual: ' + (obj["0"]));
}
