// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(NewExpression) or Type(MemberExpression) is not Object, throw
    TypeError
es5id: 11.2.2_A3_T2
description: Checking "number primitive" case
---*/

//CHECK#1
try {
  new 1;
  throw new Test262Error('#1: new 1 throw TypeError');	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1: new 1 throw TypeError');	
  }
}

//CHECK#2
try {
  var x = 1;
  new x;
  throw new Test262Error('#2: var x = 1; new x throw TypeError');	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#2: var x = 1; new x throw TypeError');	
  }
}

//CHECK#3
try {
  var x = 1;
  new x();
  throw new Test262Error('#3: var x = 1; new x() throw TypeError'); 
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#3: var x = 1; new x() throw TypeError'); 
  }
}
