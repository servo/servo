// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The pop function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.pop
description: >
    If ToUint32(length) equal zero, call the [[Put]] method  of this
    object with arguments "length" and 0 and return undefined
---*/

var obj = {};
obj.pop = Array.prototype.pop;

obj.length = NaN;
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#1: var obj = {}; obj.length = NaN; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#2: var obj = {}; obj.length = NaN; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
}

obj.length = Number.POSITIVE_INFINITY;
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#3: var obj = {}; obj.length = Number.POSITIVE_INFINITY; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 9007199254740990) {
  throw new Test262Error('#4: var obj = {}; obj.length = Number.POSITIVE_INFINITY; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 9007199254740990. Actual: ' + (obj.length));
}

obj.length = Number.NEGATIVE_INFINITY;
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#5: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#6: var obj = {}; obj.length = Number.NEGATIVE_INFINITY; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
}

obj.length = -0;
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#7: var obj = {}; obj.length = -0; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#8: var obj = {}; obj.length = -0; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
} else {
  if (1 / obj.length !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#8: var obj = {}; obj.length = -0; obj.pop = Array.prototype.pop; obj.pop(); obj.length === +0. Actual: ' + (obj.length));
  }
}

obj.length = 0.5;
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#9: var obj = {}; obj.length = 0.5; obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#10: var obj = {}; obj.length = 0.5; obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
}

obj.length = new Number(0);
var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#11: var obj = {}; obj.length = new Number(0); obj.pop = Array.prototype.pop; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#12: var obj = {}; obj.length = new Number(0); obj.pop = Array.prototype.pop; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
}
