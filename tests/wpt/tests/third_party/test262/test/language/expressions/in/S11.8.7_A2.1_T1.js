// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "in" uses GetValue
es5id: 11.8.7_A2.1_T1
description: Either Expression is not Reference or GetBase is not null
---*/

//CHECK#1
if ("MAX_VALUE" in Number !== true) {
  throw new Test262Error('#1: "MAX_VALUE" in Number === true');
}

//CHECK#2
var x = "MAX_VALUE";
if (x in Number !== true) {
  throw new Test262Error('#2: var x = "MAX_VALUE"; x in Number === true');
}

//CHECK#3
var y = Number;
if ("MAX_VALUE" in  y !== true) {
  throw new Test262Error('#3: var y = Number; "MAX_VALUE" in y === true');
}

//CHECK#4
var x = "MAX_VALUE";
var y = Number;
if (x in y !== true) {
  throw new Test262Error('#4: var x = "MAX_VALUE"; var y = Number; x in y === true');
}
