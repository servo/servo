// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of the Boolean
    constructor is the Function prototype object
esid: sec-boolean.prototype
description: Checking prototype of the Boolean constructor
---*/
assert(
  Function.prototype.isPrototypeOf(Boolean),
  'Function.prototype.isPrototypeOf(Boolean) must return true'
);
