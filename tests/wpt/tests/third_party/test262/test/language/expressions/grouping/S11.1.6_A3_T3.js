// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"This\" operator only evaluates Expression"
es5id: 11.1.6_A3_T3
description: Applying grouping operator to String
---*/

//Check for String

//CHECK#1
if (("1") !== "1") {
  throw new Test262Error('#1: ("1") === "1". Actual: ' + (("1")));
}

//CHECK#2
if (("x") !== "x") {
  throw new Test262Error('#2: ("x") === "x". Actual: ' + (("x")));
}

//CHECK#3
var x = new Number("1");
if ((x) !== x) {
  throw new Test262Error('#3: var x = new Number("1"); (x) === x. Actual: ' + ((x)));
}
