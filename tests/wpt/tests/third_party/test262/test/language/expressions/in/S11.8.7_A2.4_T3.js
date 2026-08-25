// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.8.7_A2.4_T3
description: Checking with undeclarated variables
---*/

//CHECK#1
try {
  max_value in (max_value = "MAX_VALUE", Number);
  throw new Test262Error('#1.1: max_value in (max_value = "MAX_VALUE", Number) throw ReferenceError. Actual: ' + (max_value in (max_value = "MAX_VALUE", Number)));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: max_value in (max_value = "MAX_VALUE", Number) throw ReferenceError. Actual: ' + (e));  
  }
}
