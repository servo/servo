// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Arguments : ()"
es5id: 11.2.4_A1.1_T1
description: Function is declared with no FormalParameterList
---*/

function f_arg() {
  return arguments;
}

//CHECK#1
if (f_arg().length !== 0) {
  throw new Test262Error('#1: function f_arg() {return arguments;} f_arg().length === 0. Actual: ' + (f_arg().length));
}

//CHECK#2
if (f_arg()[0] !== undefined) {
  throw new Test262Error('#2: function f_arg() {return arguments;} f_arg()[0] === undefined. Actual: ' + (f_arg()[0]));
}
