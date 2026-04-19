// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Evaluate the production ObjectLiteral: { }"
es5id: 11.1.5_A1.1
description: >
    Checking various properteis of the object defined with "var object
    = {}"
---*/

var object = {};

//CHECK#1
if (typeof object !== "object") {
  throw new Test262Error('#1: var object = {}; typeof object === "object". Actual: ' + (typeof object));
}

//CHECK#2
if (object instanceof Object !== true) {
  throw new Test262Error('#2: var object = {}; object instanceof Object === true');
}

//CHECK#3
if (object.toString !== Object.prototype.toString) {
  throw new Test262Error('#3: var object = {}; object.toString === Object.prototype.toString. Actual: ' + (object.toString));
}

//CHECK#4
if (object.toString() !== "[object Object]") {
  throw new Test262Error('#4: var object = {}; object.toString === "[object Object]". Actual: ' + (object.toString));
}
