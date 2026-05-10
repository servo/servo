// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Result(3).type is not normal, then Result(3).type must be throw.
    Throw Result(3).value as an exception
es5id: 15.1.2.1_A3.3_T4
description: Throw statement
---*/

//CHECK#1
try {
  eval("throw 1;");
  throw new Test262Error('#1.1: throw 1 must throw SyntaxError. Actual: ' + (eval("throw 1;")));
} catch(e) {
  if (e !== 1) {
    throw new Test262Error('#1.2: throw 1 must throw SyntaxError. Actual: ' + (e));
  }  
}
