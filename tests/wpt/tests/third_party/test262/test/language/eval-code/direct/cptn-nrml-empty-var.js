// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Result(3).type is normal and its completion value is empty,
    then return the value undefined
es5id: 15.1.2.1_A3.2_T2
description: Var statement
---*/

//CHECK#1
if (eval("var x = 1") !== undefined) {
  throw new Test262Error('#1: eval("var x = 1") === undefined. Actual: ' + (eval("var x = 1")));
}
