// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x < y uses GetValue
es5id: 11.8.1_A2.1_T1
description: Either Type is not Reference or GetBase is not null
---*/

//CHECK#1
if (1 < 2 !== true) {
  throw new Test262Error('#1: 1 < 2 === true');
}

//CHECK#2
var x = 1;
if (x < 2 !== true) {
  throw new Test262Error('#2: var x = 1; x < 2 === true');
}

//CHECK#3
var y = 2;
if (1 < y !== true) {
  throw new Test262Error('#3: var y = 2; 1 < y === true');
}

//CHECK#4
var x = 1;
var y = 2;
if (x < y !== true) {
  throw new Test262Error('#4: var x = 1; var y = 2; x < y === true');
}

//CHECK#5
var objectx = new Object();
var objecty = new Object();
objectx.prop = 1;
objecty.prop = 2;
if (objectx.prop < objecty.prop !== true) {
  throw new Test262Error('#5: var objectx = new Object(); var objecty = new Object(); objectx.prop = 1; objecty.prop = 2; objectx.prop < objecty.prop === true');
}
