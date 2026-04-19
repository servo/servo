// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the toString method is 0
es5id: 15.3.4.2_A11
description: Checking Function.prototype.toString.length
---*/
assert(
  Function.prototype.toString.hasOwnProperty("length"),
  'Function.prototype.toString.hasOwnProperty("length") must return true'
);

assert.sameValue(
  Function.prototype.toString.length,
  0,
  'The value of Function.prototype.toString.length is expected to be 0'
);
