// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the RegExp prototype
    object is the Object prototype
es5id: 15.10.6_A1_T1
description: Checking Object.prototype.isPrototypeOf(RegExp.prototype)
---*/
assert.sameValue(
  Object.prototype.isPrototypeOf(RegExp.prototype),
  true,
  'Object.prototype.isPrototypeOf(RegExp.prototype) must return true'
);
