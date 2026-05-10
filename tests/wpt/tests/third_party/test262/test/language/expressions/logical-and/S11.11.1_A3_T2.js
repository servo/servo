// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is false, return x
es5id: 11.11.1_A3_T2
description: Type(x) and Type(y) vary between primitive number and Number object
---*/

//CHECK#1
if ((-0 && -1) !== 0) {
  throw new Test262Error('#1.1: (-0 && -1) === 0');
} else {
  if ((1 / (-0 && -1)) !== Number.NEGATIVE_INFINITY) {
    throw new Test262Error('#1.2: (-0 && -1) === -0');
  }
}

//CHECK#2
if ((0 && new Number(-1)) !== 0) {
  throw new Test262Error('#2.1: (0 && new Number(-1)) === 0');
} else {
  if ((1 / (0 && new Number(-1))) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#2.2: (0 && new Number(-1)) === +0');
  }
}

//CHECK#3
if ((isNaN(NaN && 1)) !== true) {
  throw new Test262Error('#3: (NaN && 1) === Not-a-Number');
}
