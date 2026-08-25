// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.join
description: length = -4294967294
---*/

var obj = {};
obj.join = Array.prototype.join;
obj[0] = "x";
obj[1] = "y";
obj[2] = "z";
obj.length = -4294967294;

if (obj.join("") !== "") {
  throw new Test262Error('#1: var obj = {}; obj.join = Array.prototype.join; obj[0] = "x"; obj[1] = "y"; obj[2] = "z"; obj.length = -4294967294; obj.join("") === "". Actual: ' + (obj.join("")));
}

if (obj.length !== -4294967294) {
  throw new Test262Error('#2: var obj = {}; obj.join = Array.prototype.join; obj[0] = "x"; obj[1] = "y"; obj[2] = "z"; obj.length = -4294967294; obj.join(""); obj.length === -4294967294. Actual: ' + (obj.length));
}
