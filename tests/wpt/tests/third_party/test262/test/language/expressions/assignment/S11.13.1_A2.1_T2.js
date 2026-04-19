// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x = y uses GetValue and PutValue
es5id: 11.13.1_A2.1_T2
description: If GetBase(AssigmentExpression) is null, throw ReferenceError
---*/

//CHECK#1
try {
  x = y;
  throw new Test262Error('#1.1: x = y throw ReferenceError. Actual: ' + (x = y));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: x = y throw ReferenceError. Actual: ' + (e));  
  }
}
