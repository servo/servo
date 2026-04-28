// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(NewExpression) or Type(MemberExpression) is not Object, throw
    TypeError
es5id: 11.2.2_A3_T4
description: Checking "undefined" case
---*/

//CHECK#1
try {
  new undefined;
  throw new Test262Error('#1: new undefined throw TypeError');	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1: new undefined throw TypeError');	
  }
}

//CHECK#2
try {
  var x = undefined;
  new x;
  throw new Test262Error('#2: var x = undefined; new x throw TypeError');	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#2: var x = undefined; new x throw TypeError');	
  }
}

//CHECK#3
try {
  var x = undefined;
  new x();
  throw new Test262Error('#3: var x = undefined; new x() throw TypeError'); 
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#3: var x = undefined; new x() throw TypeError'); 
  }
}
