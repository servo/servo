// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check ToLength(length) for non Array objects
esid: sec-array.prototype.slice
description: length = 4294967296
---*/

assert.throws(RangeError, () => {
  var obj = {};
  obj.slice = Array.prototype.slice;
  obj[0] = "x";
  obj[4294967295] = "y";
  obj.length = 4294967296;
  obj.slice(0, 4294967296);
});
