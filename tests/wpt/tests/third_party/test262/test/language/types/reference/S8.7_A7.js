// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Passing arguments by reference do change values of reference to be passed
es5id: 8.7_A7
description: Add new property to original variable inside function
---*/

var n = {};
var m = n;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof m !== "object") {
  throw new Test262Error('#1: var n = {}; var m = n; typeof m === "object". Actual: ' + (typeof m));
}
//
//////////////////////////////////////////////////////////////////////////////

function populateAge(person){person.age = 50;}

populateAge(m);

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (n.age !== 50) {
  throw new Test262Error('#2: var n = {}; var m = n; function populateAge(person){person.age = 50;} populateAge(m); n.age === 50. Actual: ' + (n.age));
}

//
//////////////////////////////////////////////////////////////////////////////
