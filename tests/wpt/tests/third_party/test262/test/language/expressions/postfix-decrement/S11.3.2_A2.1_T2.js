// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x-- uses GetValue and PutValue
es5id: 11.3.2_A2.1_T2
description: If GetBase(x) is null, throw ReferenceError
---*/

//CHECK#1
try {
  x--;
  throw new Test262Error('#1.1: x-- throw ReferenceError. Actual: ' + (x--));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: x-- throw ReferenceError. Actual: ' + (e));  
  }
}
