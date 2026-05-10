// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If the eval function is called with some argument, then use a first
    argument
es5id: 15.1.2.1_A1.2_T1
description: eval("x = 1", "x = 2"), x equal 1, not 2
---*/

//CHECK#1
var x;
eval("x = 1", "x = 2");
if (x !== 1) {
  throw new Test262Error('#1: eval("x = 1", "x = 2"); x === 1. Actual: ' + (x));
}
