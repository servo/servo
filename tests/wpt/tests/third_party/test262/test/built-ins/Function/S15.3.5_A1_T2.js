// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The value of the [[Class]] property is "Function"
es5id: 15.3.5_A1_T2
description: For testing use variable f = Function()
---*/

var f = Function();

assert.sameValue(
  Object.prototype.toString.call(f),
  "[object Function]",
  'Object.prototype.toString.call(Function()) must return "[object Function]"'
);
