// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.splice
description: length is arbitrarily
---*/

var obj = {};
obj.splice = Array.prototype.splice;
obj[0] = "x";
obj[4294967295] = "y";
obj.length = 4294967296;
var arr = obj.splice(4294967295, 1);

if (arr.length !== 1) {
  throw new Test262Error('#1: var obj = {}; obj.splice = Array.prototype.splice; obj[0] = "x"; obj[4294967295] = "y"; obj.length = 4294967296; var arr = obj.splice(4294967295,1); arr.length === 1. Actual: ' + (arr.length));
}

if (obj.length !== 4294967295) {
  throw new Test262Error('#2: var obj = {}; obj.splice = Array.prototype.splice; obj[0] = "x"; obj[4294967295] = "y"; obj.length = 4294967296; var arr = obj.splice(4294967295,1); obj.length === 4294967295. Actual: ' + (obj.length));
}

if (obj[0] !== "x") {
  throw new Test262Error('#3: var obj = {}; obj.splice = Array.prototype.splice; obj[0] = "x"; obj[4294967295] = "y"; obj.length = 4294967296; var arr = obj.splice(4294967295,1); obj[0] === "x". Actual: ' + (obj[0]));
}

if (obj[4294967295] !== undefined) {
  throw new Test262Error('#4: var obj = {}; obj.splice = Array.prototype.splice; obj[0] = "x"; obj[4294967295] = "y"; obj.length = 4294967296; var arr = obj.splice(4294967295,1); obj[4294967295] === undefined. Actual: ' + (obj[4294967295]));
}

if (arr[0] !== "y") {
  throw new Test262Error('#5: var obj = {}; obj.splice = Array.prototype.splice; obj[0] = "x"; obj[4294967295] = "y"; obj.length = 4294967296; var arr = obj.splice(4294967295,1); arr[0] === "y". Actual: ' + (arr[0]));
}
