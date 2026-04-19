// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator uses GetValue
es5id: 11.14_A2.1_T1
description: Either Expression is not Reference or GetBase is not null
---*/

//CHECK#1
if ((1,2) !== 2) {
  throw new Test262Error('#1: (1,2) === 2. Actual: ' + ((1,2)));
}

//CHECK#2
var x = 1;
if ((x, 2) !== 2) {
  throw new Test262Error('#2: var x = 1; (x, 2) === 2. Actual: ' + ((x, 2)));
}

//CHECK#3
var y = 2;
if ((1, y) !== 2) {
  throw new Test262Error('#3: var y = 2; (1, y) === 2. Actual: ' + ((1, y)));
}

//CHECK#4
var x = 1;
var y = 2;
if ((x, y) !== 2) {
  throw new Test262Error('#4: var x = 1; var y = 2; (x, y) === 2. Actual: ' + ((x, y)));
}

//CHECK#5
var x = 1;
if ((x, x) !== 1) {
  throw new Test262Error('#5: var x = 1; (x, x) === 1. Actual: ' + ((x, x)));
}

//CHECK#6
var objectx = new Object();
var objecty = new Object();
objectx.prop = true;
objecty.prop = 1.1;
if ((objectx.prop = false, objecty.prop) !== objecty.prop) {
  throw new Test262Error('#6: var objectx = new Object(); var objecty = new Object(); objectx.prop = true; objecty.prop = 1; (objectx.prop = false, objecty.prop) === objecty.prop. Actual: ' + ((objectx.prop = false, objecty.prop)));
} else {
  if (objectx.prop !== false) {
    throw new Test262Error('#6: var objectx = new Object(); var objecty = new Object(); objectx.prop = true; objecty.prop = 1; objectx.prop = false, objecty.prop; objectx.prop === false');
  } 
}
