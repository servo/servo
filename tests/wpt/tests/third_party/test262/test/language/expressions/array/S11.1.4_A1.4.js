// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Evaluate the production ArrayLiteral: [ Elision, AssignmentExpression ]"
es5id: 11.1.4_A1.4
description: >
    Checking various properteis and content of the array defined with
    "var array = [,,,1,2]"
---*/

var array = [,,,1,2];

//CHECK#1
if (typeof array !== "object") {
  throw new Test262Error('#1: var array = [,,,1,2]; typeof array === "object". Actual: ' + (typeof array));
}

//CHECK#2
if (array instanceof Array !== true) {
  throw new Test262Error('#2: var array = [,,,1,2]; array instanceof Array === true');
}

//CHECK#3
if (array.toString !== Array.prototype.toString) {
  throw new Test262Error('#3: var array = [,,,1,2]; array.toString === Array.prototype.toString. Actual: ' + (array.toString));
}

//CHECK#4
if (array.length !== 5) {
  throw new Test262Error('#4: var array = [,,,1,2]; array.length === 5. Actual: ' + (array.length));
}

//CHECK#5
if (array[0] !== undefined) {
  throw new Test262Error('#5: var array = [,,,1,2]; array[0] === undefined. Actual: ' + (array[0]));
}

//CHECK#6
if (array[1] !== undefined) {
  throw new Test262Error('#6: var array = [,,,1,2]; array[1] === undefined. Actual: ' + (array[1]));
}

//CHECK#7
if (array[2] !== undefined) {
  throw new Test262Error('#7: var array = [,,,1,2]; array[2] === undefined. Actual: ' + (array[2]));
}

//CHECK#8
if (array[3] !== 1) {
  throw new Test262Error('#8: var array = [,,,1,2]; array[3] === 1. Actual: ' + (array[3]));
}

//CHECK#9
if (array[4] !== 2) {
  throw new Test262Error('#9: var array = [,,,1,2]; array[4] === 2. Actual: ' + (array[4]));
}
