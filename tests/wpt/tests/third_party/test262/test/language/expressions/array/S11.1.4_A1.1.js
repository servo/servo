// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Evaluate the production ArrayLiteral: [ ]"
es5id: 11.1.4_A1.1
description: >
    Checking various properties of the array defined with expression
    "var array = []"
---*/

var array = [];

//CHECK#1
if (typeof array !== "object") {
  throw new Test262Error('#1: var array = []; typeof array === "object". Actual: ' + (typeof array));
}

//CHECK#2
if (array instanceof Array !== true) {
  throw new Test262Error('#2: var array = []; array instanceof Array === true');
}

//CHECK#3
if (array.toString !== Array.prototype.toString) {
  throw new Test262Error('#3: var array = []; array.toString === Array.prototype.toString. Actual: ' + (array.toString));
}

//CHECK#4
if (array.length !== 0) {
  throw new Test262Error('#4: var array = []; array.length === 0. Actual: ' + (array.length));
}
