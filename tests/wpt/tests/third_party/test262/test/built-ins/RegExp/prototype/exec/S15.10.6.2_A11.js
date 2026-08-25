// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the exec method is 1
es5id: 15.10.6.2_A11
description: Checking RegExp.prototype.exec.length
---*/
assert.sameValue(
  RegExp.prototype.exec.hasOwnProperty("length"),
  true,
  'RegExp.prototype.exec.hasOwnProperty("length") must return true'
);

assert.sameValue(RegExp.prototype.exec.length, 1, 'The value of RegExp.prototype.exec.length is expected to be 1');
