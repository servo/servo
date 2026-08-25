// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Prototype]] property of the newly constructed object
    is set to the original Array prototype object, the one that
    is the initial value of Array.prototype
es5id: 15.4.1_A1.1_T3
description: Checking use isPrototypeOf
---*/

assert.sameValue(
  Array.prototype.isPrototypeOf(Array()),
  true,
  'Array.prototype.isPrototypeOf(Array()) must return true'
);
