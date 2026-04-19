// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "instanceof" uses GetValue
es5id: 11.8.6_A2.1_T2
description: If GetBase(RelationalExpression) is null, throw ReferenceError
---*/

//CHECK#1
try {
  object instanceof Object;
  throw new Test262Error('#1.1: object instanceof Object throw ReferenceError. Actual: ' + (object instanceof Object));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: object instanceof Object throw ReferenceError. Actual: ' + (e));  
  }
}
