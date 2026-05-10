// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Arguments : ()"
es5id: 11.2.4_A1.1_T2
description: Function is declared with FormalParameterList
---*/

function f_arg(x,y) {
  return arguments;
}

//CHECK#1
if (f_arg().length !== 0) {
  throw new Test262Error('#1: function f_arg(x,y) {return arguments;} f_arg().length === 0. Actual: ' + (f_arg().length));
}

//CHECK#2
if (f_arg()[0] !== undefined) {
  throw new Test262Error('#2: function f_arg(x,y) {return arguments;} f_arg()[0] === undefined. Actual: ' + (f_arg()[0]));
}

//CHECK#3
if (f_arg.length !== 2) {
  throw new Test262Error('#3: function f_arg(x,y) {return arguments;} f_arg.length === 2. Actual: ' + (f_arg.length));
}
