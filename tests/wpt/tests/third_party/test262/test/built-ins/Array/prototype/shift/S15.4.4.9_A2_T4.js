// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The shift function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.shift
description: >
    The first element of the array is removed from the array and
    returned
---*/

var obj = {};
obj["0"] = 0;
obj["3"] = 3;
obj.shift = Array.prototype.shift;

obj.length = 4;
var shift = obj.shift();
if (shift !== 0) {
  throw new Test262Error('#1: var obj = {}; obj["0"] = 0; obj["3"] = 3; obj.length = 4; obj.shift = Array.prototype.shift; obj.shift() === 0. Actual: ' + (shift));
}

if (obj.length !== 3) {
  throw new Test262Error('#2: var obj = {}; obj["0"] = 0; obj["3"] = 3; obj.length = 4; obj.shift = Array.prototype.shift; obj.shift(); obj.length === 3. Actual: ' + (obj.length));
}

var shift = obj.shift();
if (shift !== undefined) {
  throw new Test262Error('#3: var obj = {}; obj["0"] = 0; obj["3"] = 3; obj.length = 4; obj.shift = Array.prototype.shift; obj.shift(); obj.shift() === undefined. Actual: ' + (shift));
}

if (obj.length !== 2) {
  throw new Test262Error('#4: var obj = {}; obj["0"] = 0; obj["3"] = 3; obj.length = 4; obj.shift = Array.prototype.shift; obj.shift(); obj.shift(); obj.length === 2. Actual: ' + (obj.length));
}

obj.length = 1;
var shift = obj.shift();
if (shift !== undefined) {
  throw new Test262Error('#5: var obj = {}; obj["0"] = 0; obj["3"] = 3; obj.length = 4; obj.shift = Array.prototype.shift; obj.shift(); obj.shift(); obj.length = 1; obj.shift() === undefined. Actual: ' + (shift));
}

if (obj.length !== 0) {
  throw new Test262Error('#6: var obj = {}; obj["0"] = 0; obj["3"] = 3; obj.length = 4; obj.shift = Array.prototype.shift; obj.shift(); obj.shift(); obj.length = 1; obj.shift(); obj.length === 0. Actual: ' + (obj.length));
}
