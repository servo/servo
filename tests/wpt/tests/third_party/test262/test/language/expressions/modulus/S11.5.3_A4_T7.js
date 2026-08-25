// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of a ECMAScript floating-point remainder operation is
    determined by the rules of IEEE arithmetics
es5id: 11.5.3_A4_T7
description: >
    If operands neither an infinity, nor a zero, nor NaN, return x -
    truncate(x / y) * y
---*/

function truncate(x) {
  if (x > 0) {
    return Math.floor(x);
  } else {
    return Math.ceil(x);
  }
}

var x, y;

//CHECK#1
x = 1.3; 
y = 1.1;
if (x % y !== 0.19999999999999996) {
  throw new Test262Error('#1: x = 1.3; y = 1.1; x % y === 0.19999999999999996. Actual: ' + (x % y));
}

//CHECK#2
x = -1.3; 
y = 1.1; 
if (x % y !== -0.19999999999999996) {
  throw new Test262Error('#2: x = -1.3; y = 1.1; x % y === -0.19999999999999996. Actual: ' + (x % y));
}

//CHECK#3
x = 1.3; 
y = -1.1;
if (x % y !== 0.19999999999999996) {
  throw new Test262Error('#3: x = 1.3; y = -1.1; x % y === 0.19999999999999996. Actual: ' + (x % y));
}

//CHECK#4
x = -1.3; 
y = -1.1;
if (x % y !== -0.19999999999999996) {
  throw new Test262Error('#4: x = -1.3; y = -1.1; x % y === -0.19999999999999996. Actual: ' + (x % y));
}

//CHECK#5
x = 1.3; 
y = 1.1;
if (x % y !== x - truncate(x / y) * y) {
  throw new Test262Error('#5: x = 1.3; y = 1.1; x % y === x - truncate(x / y) * y. Actual: ' + (x % y));
}

//CHECK#6
x = -1.3; 
y = 1.1; 
if (x % y !== x - truncate(x / y) * y) {
  throw new Test262Error('#6: x = -1.3; y = 1.1; x % y === x - truncate(x / y) * y. Actual: ' + (x % y));
}

//CHECK#7
x = 1.3; 
y = -1.1;
if (x % y !== x - truncate(x / y) * y) {
  throw new Test262Error('#7: x = 1.3; y = -1.1; x % y === x - truncate(x / y) * y. Actual: ' + (x % y));
}

//CHECK#8
x = -1.3; 
y = -1.1;
if (x % y !== x - truncate(x / y) * y) {
  throw new Test262Error('#8: x = -1.3; y = -1.1; x % y === x - truncate(x / y) * y. Actual: ' + (x % y));
}
