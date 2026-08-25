// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.pop
description: length = 4294967297
---*/

var obj = {};
obj.pop = Array.prototype.pop;
obj[0] = "x";
obj[4294967296] = "y";
obj.length = 4294967297;

var pop = obj.pop();
if (pop !== "y") {
  throw new Test262Error('#1: var obj = {}; obj.pop = Array.prototype.pop; obj[0] = "x"; obj[4294967296] = "y"; obj.length = 4294967297; obj.pop() === "y". Actual: ' + (pop));
}

if (obj.length !== 4294967296) {
  throw new Test262Error('#2: var obj = {}; obj.pop = Array.prototype.pop; obj[0] = "x"; obj[4294967296] = "y"; obj.length = 4294967297; obj.pop(); obj.length === 4294967296. Actual: ' + (obj.length));
}

if (obj[0] !== "x") {
  throw new Test262Error('#3: var obj = {}; obj.pop = Array.prototype.pop; obj[0] = "x"; obj[4294967296] = "y"; obj.length = 4294967297; obj.pop(); obj[0] === "x". Actual: ' + (obj[0]));
}

if (obj[4294967296] !== undefined) {
  throw new Test262Error('#4: var obj = {}; obj.pop = Array.prototype.pop; obj[0] = "x"; obj[4294967296] = "y"; obj.length = 4294967297; obj.pop(); obj[4294967296] === undefined. Actual: ' + (obj[4294967296]));
}
