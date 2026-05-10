// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Result(3).type is normal and its completion value is empty,
    then return the value undefined
es5id: 15.1.2.1_A3.2_T6
description: do-while statement
---*/

//CHECK#1
if (eval("do ; while(false)") !== undefined) {
  throw new Test262Error('#1: eval("do ; while(false)") === undefined. Actual: ' + (eval("do ; while(false)")));
}
