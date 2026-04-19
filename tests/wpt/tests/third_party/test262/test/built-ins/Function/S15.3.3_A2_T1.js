// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the Function constructor
    is the Function prototype object
es5id: 15.3.3_A2_T1
description: Checking prototype of Function
---*/
assert(
  Function.prototype.isPrototypeOf(Function),
  'Function.prototype.isPrototypeOf(Function) must return true'
);
