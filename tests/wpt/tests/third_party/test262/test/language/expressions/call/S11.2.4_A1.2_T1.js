// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Arguments : (ArgumentList)"
es5id: 11.2.4_A1.2_T1
description: Function is declared with no FormalParameterList
---*/

var f_arg = function() {
  return arguments;
}

//CHECK#1
if (f_arg(1,2,3).length !== 3) {
  throw new Test262Error('#1: f_arg = function()() {return arguments;} f_arg(1,2,3).length === 3. Actual: ' + (f_arg(1,2,3).length));
}

//CHECK#2
if (f_arg(1,2,3)[0] !== 1) {
  throw new Test262Error('#1: f_arg = function()() {return arguments;} f_arg(1,2,3)[0] === 1. Actual: ' + (f_arg(1,2,3)[0]));
}

//CHECK#3
if (f_arg(1,2,3)[1] !== 2) {
  throw new Test262Error('#3: f_arg = function()() {return arguments;} f_arg(1,2,3)[1] === 2. Actual: ' + (f_arg(1,2,3)[1]));
}

//CHECK#4
if (f_arg(1,2,3)[2] !== 3) {
  throw new Test262Error('#4: f_arg = function()() {return arguments;} f_arg(1,2,3)[2] === 3. Actual: ' + (f_arg(1,2,3)[2]));
}

//CHECK#5
if (f_arg(1,2,3)[3] !== undefined) {
  throw new Test262Error('#5: f_arg = function()() {return arguments;} f_arg(1,2,3)[3] === undefined. Actual: ' + (f_arg(1,2,3)[3]));
}
