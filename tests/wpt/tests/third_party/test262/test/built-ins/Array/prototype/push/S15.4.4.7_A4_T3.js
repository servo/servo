// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.push
description: length = -1
---*/

var obj = {};
obj.push = Array.prototype.push;
obj.length = -1;

var push = obj.push("x", "y", "z");
if (push !== 3) {
  throw new Test262Error('#1: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z") === 3. Actual: ' + (push));
}

if (obj.length !== 3) {
  throw new Test262Error('#2: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z"); obj.length === 3. Actual: ' + (obj.length));
}

if (obj[4294967295] !== undefined) {
  throw new Test262Error('#3: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z"); obj[4294967295] === undefined. Actual: ' + (obj[4294967295]));
}

if (obj[4294967296] !== undefined) {
  throw new Test262Error('#4: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z"); obj[4294967296] === undefined. Actual: ' + (obj[4294967296]));
}

if (obj[4294967297] !== undefined) {
  throw new Test262Error('#5: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z"); obj[4294967297] === undefined. Actual: ' + (obj[4294967297]));
}

if (obj[0] !== "x") {
  throw new Test262Error('#3: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z"); obj[0] === "x". Actual: ' + (obj[0]));
}

if (obj[1] !== "y") {
  throw new Test262Error('#4: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z"); obj[1] === "y". Actual: ' + (obj[1]));
}

if (obj[2] !== "z") {
  throw new Test262Error('#5: var obj = {}; obj.push = Array.prototype.push; obj.length = -1; obj.push("x", "y", "z"); obj[2] === "z". Actual: ' + (obj[2]));
}
