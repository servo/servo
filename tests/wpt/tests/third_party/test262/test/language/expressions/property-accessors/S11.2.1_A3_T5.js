// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    MemberExpression calls ToObject(MemberExpression) and
    ToString(Expression). CallExpression calls ToObject(CallExpression) and
    ToString(Expression)
es5id: 11.2.1_A3_T5
description: Checking "null" case
---*/

//CHECK#1
try {
  null.toString();
  throw new Test262Error('#1.1: null.toString() throw TypeError. Actual: ' + (null.toString()));  
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1.2: null.toString() throw TypeError. Actual: ' + (e));  
  }
}

//CHECK#2
try {  
  null["toString"]();
  throw new Test262Error('#2.1: null["toString"]() throw TypeError. Actual: ' + (null["toString"]())); 
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#2.2: null["toString"]() throw TypeError. Actual: ' + (e)); 
  }
}
