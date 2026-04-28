// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.splice
description: length is arbitrarily
---*/

var obj = {};
obj.splice = Array.prototype.splice;
obj[4294967294] = "x";
obj.length = -1;
var arr = obj.splice(4294967294, 1);

if (arr.length !== 0) {
  throw new Test262Error('#1: var obj = {}; obj.splice = Array.prototype.splice; obj[4294967294] = "x"; obj.length = -1; var arr = obj.splice(4294967294,1); arr.length === 0. Actual: ' + (arr.length));
}

if (arr[0] !== undefined) {
  throw new Test262Error('#2: var obj = {}; obj.splice = Array.prototype.splice; obj[4294967294] = "x"; obj.length = 1; var arr = obj.splice(4294967294,1); arr[0] === undefined. Actual: ' + (arr[0]));
}

if (obj.length !== 0) {
  throw new Test262Error('#3: var obj = {}; obj.splice = Array.prototype.splice; obj[4294967294] = "x"; obj.length = 1; var arr = obj.splice(4294967294,1); obj.length === 0. Actual: ' + (obj.length));
}

if (obj[4294967294] !== "x") {
  throw new Test262Error('#4: var obj = {}; obj.splice = Array.prototype.splice; obj[4294967294] = "x"; obj.length = 1; var arr = obj.splice(4294967294,1); obj[4294967294] === "x". Actual: ' + (obj[4294967294]));
}
