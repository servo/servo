// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x | y uses GetValue
es5id: 11.10.3_A2.1_T3
description: If GetBase(y) is null, throw ReferenceError
---*/

//CHECK#1
try {
  1 | y;
  throw new Test262Error('#1.1: 1 | y throw ReferenceError. Actual: ' + (1 | y));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: 1 | y throw ReferenceError. Actual: ' + (e));  
  }
}
