// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If the caller supplies fewer parameter values than there are
    formal parameters, the extra formal parameters have value undefined
es5id: 10.2.1_A1
description: Calling function excluding a few parameters
---*/

//CHECK#1
function f1(a, b){
  return (b === undefined);
}
if(!(f1(1, 2) === false)){
  throw new Test262Error('#1: f1(1, 2) === false');
} else if(!(f1(1) === true)){
  throw new Test262Error('#1: f1(1) === true');
}

//CHECK#2
function f2(a, b, c){
  return (b === undefined) && (c === undefined);
}
if(!(f2(1) === true)){
  throw new Test262Error('#2: f2(1) === true');
}
