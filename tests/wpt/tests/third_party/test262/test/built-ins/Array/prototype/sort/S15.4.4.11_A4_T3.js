// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.sort
description: length = -4294967294
---*/

var obj = {};
obj.sort = Array.prototype.sort;
obj[0] = "z";
obj[1] = "y";
obj[2] = "x";
obj.length = -4294967294;

if (obj.sort() !== obj) {
  throw new Test262Error('#1: var obj = {}; obj.sort = Array.prototype.sort; obj[0] = "z"; obj[1] = "y"; obj[2] = "x"; obj.length = -4294967294; obj.sort() === obj. Actual: ' + (obj.sort()));
}

if (obj.length !== -4294967294) {
  throw new Test262Error('#2: var obj = {}; obj.sort = Array.prototype.sort; obj[0] = "z"; obj[1] = "y"; obj[2] = "x"; obj.length = -4294967294; obj.sort(); obj.length === -4294967294. Actual: ' + (obj.length));
}

if (obj[0] !== "z") {
  throw new Test262Error('#3: var obj = {}; obj.sort = Array.prototype.sort; obj[0] = "z"; obj[1] = "y"; obj[2] = "x"; obj.length = -4294967294; obj.sort(); obj[0] === "z". Actual: ' + (obj[0]));
}

if (obj[1] !== "y") {
  throw new Test262Error('#4: var obj = {}; obj.sort = Array.prototype.sort; obj[0] = "z"; obj[1] = "y"; obj[2] = "x"; obj.length = -4294967294; obj.sort(); obj[1] === "y". Actual: ' + (obj[1]));
}

if (obj[2] !== "x") {
  throw new Test262Error('#5: var obj = {}; obj.sort = Array.prototype.sort; obj[0] = "z"; obj[1] = "y"; obj[2] = "x"; obj.length = -4294967294; obj.sort(); obj[2] === "x". Actual: ' + (obj[2]));
}
