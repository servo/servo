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

obj.length = NaN;
var push = obj.push(-1);
if (push !== 1) {
  throw new Test262Error('#1: var obj = {}; obj.length = NaN; obj.push = Array.prototype.push; obj.push(-1) === 1. Actual: ' + (push));
}

if (obj.length !== 1) {
  throw new Test262Error('#2: var obj = {}; obj.length = NaN; obj.push = Array.prototype.push; obj.push(-1); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -1) {
  throw new Test262Error('#3: var obj = {}; obj.length = NaN; obj.push = Array.prototype.push; obj.push(-1); obj["0"] === -1. Actual: ' + (obj["0"]));
}

obj.length = Number.POSITIVE_INFINITY;
assert.throws(TypeError, function() {
  obj.push(-4);
});

if (obj.length !== Number.POSITIVE_INFINITY) {
  throw new Test262Error('#6: var obj = {}; obj.length = Number.POSITIVE_INFINITY; obj.push = Array.prototype.push; obj.push(-4); obj.length === Number.POSITIVE_INFINITY. Actual: ' + (obj.length));
}

if (obj[9007199254740991] !== undefined) {
  throw new Test262Error('#6: var obj = {}; obj.length = Number.POSITIVE_INFINITY; obj.push = Array.prototype.push; obj.push(-4); obj[9007199254740991] === undefined. Actual: ' + (obj["9007199254740991"]));
}

obj.length = Number.NEGATIVE_INFINITY;
var push = obj.push(-7);
if (push !== 1) {
  throw new Test262Error('#7: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.push = Array.prototype.push; obj.push(-7) === 1. Actual: ' + (push));
}

if (obj.length !== 1) {
  throw new Test262Error('#8: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.push = Array.prototype.push; obj.push(-7); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -7) {
  throw new Test262Error('#9: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.push = Array.prototype.push; obj.push(-7); obj["0"] === -7. Actual: ' + (obj["0"]));
}

obj.length = 0.5;
var push = obj.push(-10);
if (push !== 1) {
  throw new Test262Error('#10: var obj = {}; obj.length = 0.5; obj.push = Array.prototype.push; obj.push(-10) === 1. Actual: ' + (push));
}

if (obj.length !== 1) {
  throw new Test262Error('#11: var obj = {}; obj.length = 0.5; obj.push = Array.prototype.push; obj.push(-10); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -10) {
  throw new Test262Error('#12: var obj = {}; obj.length = 0.5; obj.push = Array.prototype.push; obj.push(-10); obj["0"] === -10. Actual: ' + (obj["0"]));
}

obj.length = 1.5;
var push = obj.push(-13);
if (push !== 2) {
  throw new Test262Error('#13: var obj = {}; obj.length = 1.5; obj.push = Array.prototype.push; obj.push(-13) === 2. Actual: ' + (push));
}

if (obj.length !== 2) {
  throw new Test262Error('#14: var obj = {}; obj.length = 1.5; obj.push = Array.prototype.push; obj.push(-13); obj.length === 2. Actual: ' + (obj.length));
}

if (obj["1"] !== -13) {
  throw new Test262Error('#15: var obj = {}; obj.length = 1.5; obj.push = Array.prototype.push; obj.push(-13); obj["1"] === -13. Actual: ' + (obj["1"]));
}

obj.length = new Number(0);
var push = obj.push(-16);
if (push !== 1) {
  throw new Test262Error('#16: var obj = {}; obj.length = new Number(0); obj.push = Array.prototype.push; obj.push(-16) === 1. Actual: ' + (push));
}

if (obj.length !== 1) {
  throw new Test262Error('#17: var obj = {}; obj.length = new Number(0); obj.push = Array.prototype.push; obj.push(-16); obj.length === 1. Actual: ' + (obj.length));
}

if (obj["0"] !== -16) {
  throw new Test262Error('#18: var obj = {}; obj.length = new Number(0); obj.push = Array.prototype.push; obj.push(-16); obj["0"] === -16. Actual: ' + (obj["0"]));
}
