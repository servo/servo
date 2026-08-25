// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If MemberExpression is not Object, throw TypeError
es5id: 11.2.3_A3_T2
description: Checking "number primitive" case
---*/

//CHECK#1
try {
  1();
    throw new Test262Error('#1.1: 1() throw TypeError. Actual: ' + (1()));	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1.2: 1() throw TypeError. Actual: ' + (e));	
  }
}

//CHECK#2
try {
  var x = 1;
  x();
    throw new Test262Error('#2.1: var x = 1; x() throw TypeError. Actual: ' + (x()));	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#2.2: var x = 1; x() throw TypeError. Actual: ' + (e));	
  }
}
