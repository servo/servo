// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.reverse
description: length = -4294967294
---*/

var obj = {};
obj.reverse = Array.prototype.reverse;
obj[0] = "x";
obj[1] = "y";
obj[2] = "z";
obj.length = -4294967294;

var reverse = obj.reverse();
if (reverse !== obj) {
  throw new Test262Error('#1: var obj = {}; obj.reverse = Array.prototype.reverse; obj[0] = "x"; obj[1] = "y"; obj[2] = "z"; obj.length = -4294967294; obj.reverse() === obj. Actual: ' + (reverse));
}

if (obj.length !== -4294967294) {
  throw new Test262Error('#2: var obj = {}; obj.reverse = Array.prototype.reverse; obj[0] = "x"; obj[1] = "y"; obj[2] = "z"; obj.length = -4294967294; obj.reverse(); obj.length === -4294967294. Actual: ' + (obj.length));
}

if (obj[0] !== "x") {
  throw new Test262Error('#3: var obj = {}; obj.reverse = Array.prototype.reverse; obj[0] = "x"; obj[1] = "y"; obj[2] = "z"; obj.length = -4294967294; obj.reverse(); obj[0] === "x". Actual: ' + (obj[0]));
}

if (obj[1] !== "y") {
  throw new Test262Error('#4: var obj = {}; obj.reverse = Array.prototype.reverse; obj[0] = "x"; obj[1] = "y"; obj[2] = "z"; obj.length = -4294967294; obj.reverse(); obj[1] === "y". Actual: ' + (obj[1]));
}

if (obj[2] !== "z") {
  throw new Test262Error('#5: var obj = {}; obj.reverse = Array.prototype.reverse; obj[0] = "x"; obj[1] = "y"; obj[2] = "z"; obj.length = -4294967294; obj.reverse(); obj[2] === "z". Actual: ' + (obj[2]));
}
