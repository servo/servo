// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.8.7_A2.4_T1
description: Checking with "="
---*/

//CHECK#1 
var NUMBER = 0;
if ((NUMBER = Number, "MAX_VALUE") in NUMBER !== true) {
  throw new Test262Error('#1: var NUMBER = 0; (NUMBER = Number, "MAX_VALUE") in NUMBER === true');
}

//CHECK#2
var max_value = "MAX_VALUE"; 
if (max_value in (max_value = "none", Number) !== true) {
  throw new Test262Error('#2: var max_value = "MAX_VALUE"; max_value in (max_value = "none", Number) === true');
}
