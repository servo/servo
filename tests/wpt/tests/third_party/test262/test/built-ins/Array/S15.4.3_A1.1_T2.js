// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The value of the internal [[Prototype]] property of
    the Array constructor is the Function prototype object
es5id: 15.4.3_A1.1_T2
description: Function.prototype.toString = Object.prototype.toString
---*/

Function.prototype.toString = Object.prototype.toString;

assert.sameValue(
  Array.toString(),
  "[object Function]",
  'Array.toString() must return "[object Function]"'
);
