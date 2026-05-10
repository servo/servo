// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check Prefix Increment Operator for automatic semicolon insertion
es5id: 7.9_A5.2_T1
description: Try use Variable1 \n ++Variable2 construction
---*/

//CHECK#1
var x = 0;
var y = 0;
x
++y
if (x !== 0) {
  throw new Test262Error('#1: Check Prefix Increment Operator for automatic semicolon insertion');
} else {
  if (y !== 1) {
    throw new Test262Error('#2: Check Prefix Increment Operator for automatic semicolon insertion');
  }
}
