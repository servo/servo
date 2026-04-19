// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If either x or y is +0 and the other is -0, return true
es5id: 11.8.4_A4.4
description: Checking all combinations
---*/

//CHECK#1
if ((0 >= 0) !== true) {
  throw new Test262Error('#1: (0 >= 0) === true');
}

//CHECK#2
if ((-0 >= -0) !== true) {
  throw new Test262Error('#2: (-0 >= -0) === true');
}

//CHECK#3
if ((+0 >= -0) !== true) {
  throw new Test262Error('#3: (+0 >= -0) === true');
}

//CHECK#4
if ((-0 >= +0) !== true) {
  throw new Test262Error('#4: (-0 >= +0) === true');
}
