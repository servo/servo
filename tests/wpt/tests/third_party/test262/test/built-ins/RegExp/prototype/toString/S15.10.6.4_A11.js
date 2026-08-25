// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the toString method is 1
es5id: 15.10.6.4_A11
description: Checking RegExp.prototype.toString.length
---*/
assert.sameValue(
  RegExp.prototype.toString.hasOwnProperty("length"),
  true,
  'RegExp.prototype.toString.hasOwnProperty("length") must return true'
);

assert.sameValue(
  RegExp.prototype.toString.length,
  0,
  'The value of RegExp.prototype.toString.length is expected to be 0'
);
