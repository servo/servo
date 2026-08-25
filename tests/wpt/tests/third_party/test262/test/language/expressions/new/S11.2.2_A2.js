// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "new" uses GetValue
es5id: 11.2.2_A2
description: >
    If GetBase(NewExpression) or GetBase(MemberExpression) is null,
    throw ReferenceError
---*/

//CHECK#1
try {
  new x;
  throw new Test262Error('#1.1: new x throw ReferenceError. Actual: ' + (new x));  
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#1.2: new x throw ReferenceError. Actual: ' + (e));  
  }
}

//CHECK#2
try {
  new x();
  throw new Test262Error('#2: new x() throw ReferenceError'); 
}
catch (e) {
  if ((e instanceof ReferenceError) !== true) {
    throw new Test262Error('#2: new x() throw ReferenceError'); 
  }
}
