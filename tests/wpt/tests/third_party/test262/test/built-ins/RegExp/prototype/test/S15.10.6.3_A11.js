// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the test method is 1
es5id: 15.10.6.3_A11
description: Checking RegExp.prototype.test.length
---*/
assert.sameValue(
  RegExp.prototype.test.hasOwnProperty("length"),
  true,
  'RegExp.prototype.test.hasOwnProperty("length") must return true'
);

assert.sameValue(RegExp.prototype.test.length, 1, 'The value of RegExp.prototype.test.length is expected to be 1');
