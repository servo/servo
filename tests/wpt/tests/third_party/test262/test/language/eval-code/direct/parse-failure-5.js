// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Result(3).type is not normal, then Result(3).type must be throw.
    Throw Result(3).value as an exception
es5id: 15.1.2.1_A3.3_T3
description: Return statement
---*/

//CHECK#1
try {
  eval("return;");
  throw new Test262Error('#1.1: return must throw SyntaxError. Actual: ' + (eval("return;")));
} catch(e) {
  if ((e instanceof SyntaxError) !== true) {
    throw new Test262Error('#1.2: return must throw SyntaxError. Actual: ' + (e));
  }  
}

//CHECK#2

function f() {  eval("return;"); };

try {
  f();      
  throw new Test262Error('#2.1: return must throw SyntaxError. Actual: ' + (f()));
} catch(e) {
  if ((e instanceof SyntaxError) !== true) {
    throw new Test262Error('#2.2: return must throw SyntaxError. Actual: ' + (e));
  }  
}
