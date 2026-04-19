// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x = y uses GetValue and PutValue
es5id: 11.13.1_A2.1_T1
description: Either AssigmentExpression is not Reference or GetBase is not null
---*/

//CHECK#1
x = 1;
if (x !== 1) {
  throw new Test262Error('#1: x = 1; x === 1. Actual: ' + (x));
}

//CHECK#2
var x = 1;
if (x !== 1) {
  throw new Test262Error('#2: var x = 1; x === 1. Actual: ' + (x));
}

//CHECK#3
y = 1;
x = y;
if (x !== 1) {
  throw new Test262Error('#3: y = 1; x = y; x === 1. Actual: ' + (x));
}

//CHECK#4
var y = 1;
var x = y;
if (x !== 1) {
  throw new Test262Error('#4: var y = 1; var x = y; x === 1. Actual: ' + (x));
}

//CHECK#5
var objectx = new Object();
var objecty = new Object();
objecty.prop = 1.1;
objectx.prop = objecty.prop;
if (objectx.prop !== objecty.prop) {
  throw new Test262Error('#5: var objectx = new Object(); var objecty = new Object(); objecty.prop = 1; objectx.prop = objecty.prop; objectx.prop === objecty.prop. Actual: ' + (objectx.prop));
} else {
  if (objectx === objecty) {
    throw new Test262Error('#5: var objectx = new Object(); var objecty = new Object(); objecty.prop = 1; objectx.prop = objecty.prop; objectx !== objecty');
  } 
}
