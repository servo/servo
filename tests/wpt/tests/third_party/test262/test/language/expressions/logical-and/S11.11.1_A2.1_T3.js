// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x && y uses GetValue
es5id: 11.11.1_A2.1_T3
description: >
    If ToBoolean(x) is true and GetBase(y) is null, throw
    ReferenceError
---*/

//CHECK#1
try {
  true && y;
  throw new Test262Error('#1.1: true && y throw ReferenceError. Actual: ' + (true && y));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: true && y throw ReferenceError. Actual: ' + (e));  
  }
}
