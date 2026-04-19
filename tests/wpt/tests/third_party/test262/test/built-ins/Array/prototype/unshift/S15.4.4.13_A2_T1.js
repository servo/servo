// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The unshift function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.unshift
description: >
    The arguments are prepended to the start of the array, such that
    their order within the array is the same as the order in which
    they appear in  the argument list
---*/

var obj = {};
obj.unshift = Array.prototype.unshift;

if (obj.length !== undefined) {
  throw new Test262Error('#0: var obj = {}; obj.length === undefined. Actual: ' + (obj.length));
} else {
    var unshift = obj.unshift(-1);
  if (unshift !== 1) {
    throw new Test262Error('#1: var obj = {}; obj.unshift = Array.prototype.unshift; obj.unshift(-1) === 1. Actual: ' + (unshift));
  }
    if (obj.length !== 1) {
    throw new Test262Error('#2: var obj = {}; obj.unshift = Array.prototype.unshift; obj.unshift(-1); obj.length === 1. Actual: ' + (obj.length));
  }
    if (obj["0"] !== -1) {
    throw new Test262Error('#3: var obj = {}; obj.unshift = Array.prototype.unshift; obj.unshift(-1); obj["0"] === -1. Actual: ' + (obj["0"]));
  }
}

obj.length = undefined;
var unshift = obj.unshift(-4);
if (unshift !== 1) {
  throw new Test262Error('#4: var obj = {}; obj.length = undefined; obj.unshift = Array.prototype.unshift; obj.unshift(-4) === 1. Actual: ' + (unshift));
}

if (obj.length !== 1) {
  throw new Test262Error('#5: var obj = {}; obj.length = undefined; obj.unshift = Array.prototype.unshift; obj.unshift(-4); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -4) {
  throw new Test262Error('#6: var obj = {}; obj.length = undefined; obj.unshift = Array.prototype.unshift; obj.unshift(-4); obj["0"] === -4. Actual: ' + (obj["0"]));
}

obj.length = null
var unshift = obj.unshift(-7);
if (unshift !== 1) {
  throw new Test262Error('#7: var obj = {}; obj.length = null; obj.unshift = Array.prototype.unshift; obj.unshift(-7) === 1. Actual: ' + (unshift));
}

if (obj.length !== 1) {
  throw new Test262Error('#8: var obj = {}; obj.length = null; obj.unshift = Array.prototype.unshift; obj.unshift(-7); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -7) {
  throw new Test262Error('#9: var obj = {}; obj.length = null; obj.unshift = Array.prototype.unshift; obj.unshift(-7); obj["0"] === -7. Actual: ' + (obj["0"]));
}
