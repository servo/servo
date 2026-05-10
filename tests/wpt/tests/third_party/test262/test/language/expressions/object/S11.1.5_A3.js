// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Evaluate the production ObjectLiteral: { PropertyNameAndValueList }"
es5id: 11.1.5_A3
description: >
    Creating the object defined with "var object = {0 : 1, "1" : "x",
    o : {}}"
---*/

var object = {0 : 1, "1" : "x", o : {}};

//CHECK#1
if (object[0] !== 1) {
  throw new Test262Error('#1: var object = {0 : 1; "1" : "x"; o : {}}; object[0] === 1. Actual: ' + (object[0]));
}

//CHECK#2
if (object["1"] !== "x") {
  throw new Test262Error('#2: var object = {0 : 1; "1" : "x"; o : {}}; object["1"] === "x". Actual: ' + (object["1"]));
}

//CHECK#3
if (typeof object.o !== "object") {
  throw new Test262Error('#1: var object = {0 : 1; "1" : "x"; o : {}}; typeof object.o === "object". Actual: ' + (typeof object.o));
}
