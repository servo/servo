// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.pop
description: length = -1
---*/

var obj = {};
obj.pop = Array.prototype.pop;
obj[4294967294] = "x";
obj.length = -1;

var pop = obj.pop();
if (pop !== undefined) {
  throw new Test262Error('#1: var obj = {}; obj.pop = Array.prototype.pop; obj[4294967294] = "x"; obj.length = -1; obj.pop() === undefined. Actual: ' + (pop));
}

if (obj.length !== 0) {
  throw new Test262Error('#2: var obj = {}; obj.pop = Array.prototype.pop; obj[4294967294] = "x"; obj.length = -1; obj.pop(); obj.length === 0. Actual: ' + (obj.length));
}

if (obj[4294967294] !== "x") {
  throw new Test262Error('#3: var obj = {}; obj.pop = Array.prototype.pop; obj[4294967294] = "x"; obj.length = -1; obj.pop(); obj[4294967294] === "x". Actual: ' + (obj[4294967294]));
}
