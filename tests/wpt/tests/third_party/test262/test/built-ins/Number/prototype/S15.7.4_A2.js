// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the Number
    prototype object is the Object prototype object
es5id: 15.7.4_A2
description: Checking Object.prototype.isPrototypeOf(Number.prototype)
---*/
assert(
  Object.prototype.isPrototypeOf(Number.prototype),
  'Object.prototype.isPrototypeOf(Number.prototype) must return true'
);
