// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The splice function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.splice
description: >
    If start is negative, use max(start + length, 0).  If deleteCount
    is negative, use 0
---*/

var obj = {
  0: 0,
  1: 1
};
obj.length = 2;
obj.splice = Array.prototype.splice;
var arr = obj.splice(-2, -1, 2, 3);

arr.getClass = Object.prototype.toString;
if (arr.getClass() !== "[object " + "Array" + "]") {
  throw new Test262Error('#0: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); arr is Array object. Actual: ' + (arr.getClass()));
}

if (arr.length !== 0) {
  throw new Test262Error('#1: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); arr.length === 0. Actual: ' + (arr.length));
}

if (obj.length !== 4) {
  throw new Test262Error('#2: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); obj.length === 4. Actual: ' + (obj.length));
}

if (obj[0] !== 2) {
  throw new Test262Error('#3: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); obj[0] === 2. Actual: ' + (obj[0]));
}

if (obj[1] !== 3) {
  throw new Test262Error('#4: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); obj[1] === 3. Actual: ' + (obj[1]));
}

if (obj[2] !== 0) {
  throw new Test262Error('#5: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); obj[2] === 0. Actual: ' + (obj[2]));
}

if (obj[3] !== 1) {
  throw new Test262Error('#6: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); obj[3] === 1. Actual: ' + (obj[3]));
}

if (obj[4] !== undefined) {
  throw new Test262Error('#7: var obj = {0:0,1:1}; obj.length = 2; obj.splice = Array.prototype.splice; var arr = obj.splice(-2,-1,2,3); obj[4] === undefined. Actual: ' + (obj[4]));
}
