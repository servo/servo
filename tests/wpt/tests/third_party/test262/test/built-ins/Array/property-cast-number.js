// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property name P (in the form of a string value) is an array index
    if and only if ToString(ToUint32(P)) is equal to P and ToUint32(P) is not equal to 2^32 - 1
es5id: 15.4_A1.1_T3
description: Checking for number primitive
---*/

var x = [];
x[4294967296] = 1;
assert.sameValue(x[0], undefined, 'The value of x[0] is expected to equal undefined');
assert.sameValue(x["4294967296"], 1, 'The value of x["4294967296"] is expected to be 1');

var y = [];
y[4294967297] = 1;
if (y[1] !== undefined) {
  throw new Test262Error('#3: y = []; y[4294967297] = 1; y[1] === undefined. Actual: ' + (y[1]));
}

//CHECK#4
if (y["4294967297"] !== 1) {
  throw new Test262Error('#4: y = []; y[4294967297] = 1; y["4294967297"] === 1. Actual: ' + (y["4294967297"]));
}

//CHECK#5
var z = [];
z[1.1] = 1;
if (z[1] !== undefined) {
  throw new Test262Error('#5: z = []; z[1.1] = 1; z[1] === undefined. Actual: ' + (z[1]));
}

//CHECK#6
if (z["1.1"] !== 1) {
  throw new Test262Error('#6: z = []; z[1.1] = 1; z["1.1"] === 1. Actual: ' + (z["1.1"]));
}
