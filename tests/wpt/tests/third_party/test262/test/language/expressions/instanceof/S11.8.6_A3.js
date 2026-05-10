// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ShiftExpression is not an object, throw TypeError
es5id: 11.8.6_A3
description: Checking all the types of primitives
---*/

//CHECK#1
try {
  true instanceof true;
  throw new Test262Error('#1: true instanceof true throw TypeError');  
}
catch (e) {
  if (e instanceof TypeError !== true) {
    throw new Test262Error('#1: true instanceof true throw TypeError');  
  }
}

//CHECK#2
try {
  1 instanceof 1;
  throw new Test262Error('#2: 1 instanceof 1 throw TypeError');  
}
catch (e) {
  if (e instanceof TypeError !== true) {
    throw new Test262Error('#2: 1 instanceof 1 throw TypeError');  
  }
}

//CHECK#3
try {
  "string" instanceof "string";
  throw new Test262Error('#3: "string" instanceof "string" throw TypeError');  
}
catch (e) {
  if (e instanceof TypeError !== true) {
    throw new Test262Error('#3: "string" instanceof "string" throw TypeError');  
  }
}

//CHECK#4
try {
  undefined instanceof undefined;
  throw new Test262Error('#4: undefined instanceof undefined throw TypeError');  
}
catch (e) {
  if (e instanceof TypeError !== true) {
    throw new Test262Error('#4: undefined instanceof undefined throw TypeError');  
  }
}

//CHECK#5
try {
  null instanceof null;
  throw new Test262Error('#5: null instanceof null throw TypeError');  
}
catch (e) {
  if (e instanceof TypeError !== true) {
    throw new Test262Error('#5: null instanceof null throw TypeError');  
  }
}
