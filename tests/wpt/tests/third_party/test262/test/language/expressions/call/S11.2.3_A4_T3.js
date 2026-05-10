// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If MemberExpression does not implement the internal [[Call]] method,
    throw TypeError
es5id: 11.2.3_A4_T3
description: Checking String object case
---*/

//CHECK#1
try {
  new String("1")();
  throw new Test262Error('#1.1: new String("1")() throw TypeError. Actual: ' + (new String("1")()));	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1.2: new String("1")() throw TypeError. Actual: ' + (e));	
  }
}

//CHECK#2
try {
  var x = new String("1");
  x();
  throw new Test262Error('#2.1: var x = new String("1"); x() throw TypeError. Actual: ' + (x()));	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#2.2: var x = new String("1"); x() throw TypeError. Actual: ' + (e));	
  }
}
