// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToUint32(length) for non Array objects
esid: sec-array.prototype.slice
description: length = -1
---*/

var obj = {};
obj.slice = Array.prototype.slice;
obj[4294967294] = "x";
obj.length = -1;
var arr = obj.slice(4294967294, 4294967295);

if (arr.length !== 0) {
  throw new Test262Error('#1: var obj = {}; obj.slice = Array.prototype.slice; obj[4294967294] = "x"; obj.length = 4294967295; var arr = obj.slice(4294967294,4294967295); arr.length === 0. Actual: ' + (arr.length));
}

if (arr[0] !== undefined) {
  throw new Test262Error('#3: var obj = {}; obj.slice = Array.prototype.slice; obj[4294967294] = "x"; obj.length = 4294967295; var arr = obj.slice(4294967294,4294967295); arr[0] === undefined. Actual: ' + (arr[0]));
}
