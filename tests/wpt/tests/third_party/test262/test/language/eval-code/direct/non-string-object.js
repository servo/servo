// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is not a string value, return x
es5id: 15.1.2.1_A1.1_T2
description: Checking all object
---*/

//CHECK#1
var x = {};
if (eval(x) !== x) {
  throw new Test262Error('#1: x = {}; eval(x) === x. Actual: ' + (eval(x)));
}

//CHECK#2
x = new Number(1);
if (eval(x) !== x) {
  throw new Test262Error('#2: x = new Number(1); eval(x) === x. Actual: ' + (eval(x)));
}

//CHECK#3
x = new Boolean(true);
if (eval(x) !== x) {
  throw new Test262Error('#3: x = new Boolean(true); eval(x) === x. Actual: ' + (eval(x)));
}

//CHECK#4
x = new String("1+1");
if (eval(x) !== x) {
  throw new Test262Error('#4: x = new String("1"); eval(x) === x. Actual: ' + (eval(x)));
}
