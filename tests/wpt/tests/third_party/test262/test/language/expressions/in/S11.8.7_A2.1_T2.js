// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "in" uses GetValue
es5id: 11.8.7_A2.1_T2
description: If GetBase(RelationalExpression) is null, throw ReferenceError
---*/

//CHECK#1
try {
  MAX_VALUE in Number;
  throw new Test262Error('#1.1: MAX_VALUE in Number throw ReferenceError. Actual: ' + (MAX_VALUE in Number));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: MAX_VALUE in Number throw ReferenceError. Actual: ' + (e));  
  }
}
