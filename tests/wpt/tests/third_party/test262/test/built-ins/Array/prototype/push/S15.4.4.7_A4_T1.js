// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.push
description: length = 4294967296
---*/

var obj = {};
obj.push = Array.prototype.push;
obj.length = 4294967296;

var push = obj.push("x", "y", "z");
if (push !== 4294967299) {
  throw new Test262Error('#1: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z") === 4294967299. Actual: ' + (push));
}

if (obj.length !== 4294967299) {
  throw new Test262Error('#2: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z"); obj.length === 4294967299. Actual: ' + (obj.length));
}

if (obj[0] !== undefined) {
  throw new Test262Error('#3: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z"); obj[0] === undefined. Actual: ' + (obj[0]));
}

if (obj[1] !== undefined) {
  throw new Test262Error('#4: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z"); obj[1] === undefined. Actual: ' + (obj[1]));
}

if (obj[2] !== undefined) {
  throw new Test262Error('#5: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z"); obj[2] === undefined. Actual: ' + (obj[2]));
}

if (obj[4294967296] !== "x") {
  throw new Test262Error('#6: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z"); obj[4294967296] === "x". Actual: ' + (obj[4294967296]));
}

if (obj[4294967297] !== "y") {
  throw new Test262Error('#7: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z"); obj[4294967297] === "y". Actual: ' + (obj[4294967297]));
}

if (obj[4294967298] !== "z") {
  throw new Test262Error('#8: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push("x", "y", "z"); obj[4294967298] === "z". Actual: ' + (obj[4294967298]));
}

var obj = {};
obj.push = Array.prototype.push;
obj.length = 4294967296;

var push = obj.push();
if (push !== 4294967296) {
  throw new Test262Error('#9: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push() === 4294967296. Actual: ' + (push));
}

if (obj.length !== 4294967296) {
  throw new Test262Error('#10: var obj = {}; obj.push = Array.prototype.push; obj.length = 4294967296; obj.push(); obj.length === 4294967296. Actual: ' + (obj.length));
}
