// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Evaluate the production ObjectLiteral: { NumericLiteral :
    AssignmentExpression}
es5id: 11.1.5_A1.2
description: >
    Checking various properteis and contents of the object defined
    with "var object = {1 : true}"
---*/

var object = {1 : true};

//CHECK#1
if (typeof object !== "object") {
  throw new Test262Error('#1: var object = {1 : true}; typeof object === "object". Actual: ' + (typeof object));
}

//CHECK#2
if (object instanceof Object !== true) {
  throw new Test262Error('#2: var object = {1 : true}; object instanceof Object === true');
}

//CHECK#3
if (object.toString !== Object.prototype.toString) {
  throw new Test262Error('#3: var object = {1 : true}; object.toString === Object.prototype.toString. Actual: ' + (object.toString));
}

//CHECK#4
if (object[1] !== true) {
  throw new Test262Error('#4: var object = {1 : true}; object[1] === true');
}

//CHECK#5
if (object["1"] !== true) {
  throw new Test262Error('#5: var object = {1 : true}; object["1"] === true');
}
