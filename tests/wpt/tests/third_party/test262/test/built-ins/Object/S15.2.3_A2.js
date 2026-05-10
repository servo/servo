// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the Object constructor
    is the Function prototype object
es5id: 15.2.3_A2
description: Checking Function.prototype.isPrototypeOf(Object)
---*/
assert(
  !!Function.prototype.isPrototypeOf(Object),
  'The value of !!Function.prototype.isPrototypeOf(Object) is expected to be true'
);
