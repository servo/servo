// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ShiftExpression is not an object, throw TypeError
es5id: 11.8.7_A3
description: Checking all the types of primitives
---*/

//CHECK#1
try {
  "toString" in true;
  throw new Test262Error('#1: "toString" in true throw TypeError');  
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1: "toString" in true throw TypeError');  
  }
}

//CHECK#2
try {
  "MAX_VALUE" in 1;
  throw new Test262Error('#2: "MAX_VALUE" in 1 throw TypeError');  
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#2: "MAX_VALUE" in 1 throw TypeError');  
  }
}

//CHECK#3
try {
  "length" in "string";
  throw new Test262Error('#3: "length" in "string" throw TypeError');  
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#3: "length" in "string" throw TypeError');  
  }
}

//CHECK#4
try {
  "toString" in undefined;
  throw new Test262Error('#4: "toString" in undefined throw TypeError');  
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#4: "toString" in undefined throw TypeError');  
  }
}

//CHECK#5
try {
  "toString" in null;
  throw new Test262Error('#5: "toString" in null throw TypeError');  
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#5: "toString" in null throw TypeError');  
  }
}
