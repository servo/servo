// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x !== y uses GetValue
es5id: 11.9.5_A2.1_T2
description: If GetBase(x) is null, throw ReferenceError
---*/

//CHECK#1
try {
  x !== 1;
  throw new Test262Error('#1.1: x !== 1 throw ReferenceError. Actual: ' + (x !== 1));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: x !== 1 throw ReferenceError. Actual: ' + (e));  
  }
}
