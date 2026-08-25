// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.push
description: length = 4294967295
---*/

var obj = {};
obj.push = Array.prototype.push;
obj.length = 4294967295;

var push = obj.push("x", "y", "z");
if (push !== 4294967298) {
  throw new Test262Error('#1: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967295; obj.push("x", "y", "z") === 4294967298. Actual: ' + (push));
}

if (obj.length !== 4294967298) {
  throw new Test262Error('#2: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967295; obj.push("x", "y", "z"); obj.length === 4294967298. Actual: ' + (obj.length));
}

if (obj[4294967295] !== "x") {
  throw new Test262Error('#3: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967295; obj.push("x", "y", "z"); obj[4294967295] === "x". Actual: ' + (obj[4294967295]));
}

if (obj[4294967296] !== "y") {
  throw new Test262Error('#4: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967295; obj.push("x", "y", "z"); obj[4294967296] === "y". Actual: ' + (obj[4294967296]));
}

if (obj[4294967297] !== "z") {
  throw new Test262Error('#5: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967295; obj.push("x", "y", "z"); obj[4294967297] === "z". Actual: ' + (obj[4294967297]));
}
