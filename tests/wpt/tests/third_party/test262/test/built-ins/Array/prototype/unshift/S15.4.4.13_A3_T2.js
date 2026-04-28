// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.unshift
description: length = -4294967295
---*/

var obj = {};
obj.unshift = Array.prototype.unshift;
obj[0] = "";
obj.length = -4294967295;

var unshift = obj.unshift("x", "y", "z");
if (unshift !== 3) {
  throw new Test262Error('#1: var obj = {}; obj.unshift = Array.prototype.unshift; obj[0] = ""; obj.length = -4294967295; obj.unshift("x", "y", "z") === 3. Actual: ' + (unshift));
}

if (obj.length !== 3) {
  throw new Test262Error('#2: var obj = {}; obj.unshift = Array.prototype.unshift; obj[0] = ""; obj.length = -4294967295; obj.unshift("x", "y", "z"); obj.length === 3. Actual: ' + (obj.length));
}

if (obj[0] !== "x") {
  throw new Test262Error('#3: var obj = {}; obj.unshift = Array.prototype.unshift; obj[0] = ""; obj.length = -4294967295; obj.unshift("x", "y", "z"); obj[0] === "x". Actual: ' + (obj[0]));
}

if (obj[1] !== "y") {
  throw new Test262Error('#4: var obj = {}; obj.unshift = Array.prototype.unshift; obj[0] = ""; obj.length = -4294967295; obj.unshift("x", "y", "z"); obj[1] === "y". Actual: ' + (obj[1]));
}

if (obj[2] !== "z") {
  throw new Test262Error('#5: var obj = {}; obj.unshift = Array.prototype.unshift; obj[0] = ""; obj.length = -4294967295; obj.unshift("x", "y", "z"); obj[2] === "z". Actual: ' + (obj[2]));
}

if (obj[3] !== undefined) {
  throw new Test262Error('#6: var obj = {}; obj.unshift = Array.prototype.unshift; obj[0] = ""; obj.length = -4294967295; obj.unshift("x", "y", "z"); obj[3] === undefined. Actual: ' + (obj[3]));
}
