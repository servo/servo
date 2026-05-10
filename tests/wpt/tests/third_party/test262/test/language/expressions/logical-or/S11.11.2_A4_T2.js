// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return x
es5id: 11.11.2_A4_T2
description: Type(x) and Type(y) vary between primitive number and Number object
---*/

//CHECK#1
if ((-1 || 1) !== -1) {
  throw new Test262Error('#1: (-1 || 1) === -1');
}

//CHECK#2
if ((1 || new Number(0)) !== 1) {
  throw new Test262Error('#2: (1 || new Number(0)) === 1');
} 

//CHECK#3
if ((-1 || NaN) !== -1) {
  throw new Test262Error('#3: (-1 || NaN) === -1');
}

//CHECK#4
var x = new Number(-1);
if ((x || new Number(0)) !== x) {
  throw new Test262Error('#4: (var x = new Number(-1); (x || new Number(-1)) === x');
}

//CHECK#5
var x = new Number(NaN);
if ((x || new Number(1)) !== x) {
  throw new Test262Error('#5: (var x = new Number(NaN); (x || new Number(1)) === x');
}

//CHECK#6
var x = new Number(0);
if ((x || new Number(NaN)) !== x) {
  throw new Test262Error('#6: (var x = new Number(0); (x || new Number(NaN)) === x');
}
