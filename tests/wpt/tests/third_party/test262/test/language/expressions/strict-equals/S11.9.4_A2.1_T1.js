// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x === y uses GetValue
es5id: 11.9.4_A2.1_T1
description: Either Type is not Reference or GetBase is not null
---*/

//CHECK#1
if (!(1 === 1)) {
  throw new Test262Error('#1: 1 === 1');
}

//CHECK#2
var x = 1;
if (!(x === 1)) {
  throw new Test262Error('#2: var x = 1; x === 1');
}

//CHECK#3
var y = 1;
if (!(1 === y)) {
  throw new Test262Error('#3: var y = 1; 1 === y');
}

//CHECK#4
var x = 1;
var y = 1;
if (!(x === y)) {
  throw new Test262Error('#4: var x = 1; var y = 1; x === y');
}

//CHECK#5
var objectx = new Object();
var objecty = new Object();
objectx.prop = 1;
objecty.prop = 1;
if (!(objectx.prop === objecty.prop)) {
  throw new Test262Error('#5: var objectx = new Object(); var objecty = new Object(); objectx.prop = 1; objecty.prop = 1; objectx.prop === objecty.prop');
}
