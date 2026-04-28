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

obj.length = NaN;
var unshift = obj.unshift(-1);
if (unshift !== 1) {
  throw new Test262Error('#1: var obj = {}; obj.length = NaN; obj.unshift = Array.prototype.unshift; obj.unshift(-1) === 1. Actual: ' + (unshift));
}

if (obj.length !== 1) {
  throw new Test262Error('#2: var obj = {}; obj.length = NaN; obj.unshift = Array.prototype.unshift; obj.unshift(-1); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -1) {
  throw new Test262Error('#3: var obj = {}; obj.length = NaN; obj.unshift = Array.prototype.unshift; obj.unshift(-1); obj["0"] === -1. Actual: ' + (obj["0"]));
}

obj.length = Number.NEGATIVE_INFINITY;
var unshift = obj.unshift(-7);
if (unshift !== 1) {
  throw new Test262Error('#7: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.unshift = Array.prototype.unshift; obj.unshift(-7) === 1. Actual: ' + (unshift));
}

if (obj.length !== 1) {
  throw new Test262Error('#8: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.unshift = Array.prototype.unshift; obj.unshift(-7); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -7) {
  throw new Test262Error('#9: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.unshift = Array.prototype.unshift; obj.unshift(-7); obj["0"] === -7. Actual: ' + (obj["0"]));
}

obj.length = 0.5;
var unshift = obj.unshift(-10);
if (unshift !== 1) {
  throw new Test262Error('#10: var obj = {}; obj.length = 0.5; obj.unshift = Array.prototype.unshift; obj.unshift(-10) === 1. Actual: ' + (unshift));
}

if (obj.length !== 1) {
  throw new Test262Error('#11: var obj = {}; obj.length = 0.5; obj.unshift = Array.prototype.unshift; obj.unshift(-10); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -10) {
  throw new Test262Error('#12: var obj = {}; obj.length = 0.5; obj.unshift = Array.prototype.unshift; obj.unshift(-10); obj["0"] === -10. Actual: ' + (obj["0"]));
}

obj.length = 1.5;
var unshift = obj.unshift(-13);
if (unshift !== 2) {
  throw new Test262Error('#13: var obj = {}; obj.length = 1.5; obj.unshift = Array.prototype.unshift; obj.unshift(-13) === 2. Actual: ' + (unshift));
}

if (obj.length !== 2) {
  throw new Test262Error('#14: var obj = {}; obj.length = 1.5; obj.unshift = Array.prototype.unshift; obj.unshift(-13); obj.length === 2. Actual: ' + (obj.length));
}

if (obj["0"] !== -13) {
  throw new Test262Error('#15: var obj = {}; obj.length = 1.5; obj.unshift = Array.prototype.unshift; obj.unshift(-13); obj["0"] === -13. Actual: ' + (obj["0"]));
}

obj.length = new Number(0);
var unshift = obj.unshift(-16);
if (unshift !== 1) {
  throw new Test262Error('#16: var obj = {}; obj.length = new Number(0); obj.unshift = Array.prototype.unshift; obj.unshift(-16) === 1. Actual: ' + (unshift));
}

if (obj.length !== 1) {
  throw new Test262Error('#17: var obj = {}; obj.length = new Number(0); obj.unshift = Array.prototype.unshift; obj.unshift(-16); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -16) {
  throw new Test262Error('#18: var obj = {}; obj.length = new Number(0); obj.unshift = Array.prototype.unshift; obj.unshift(-16); obj["0"] === -16. Actual: ' + (obj["0"]));
}
