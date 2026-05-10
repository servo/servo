// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If is x greater than y and these values are both finite non-zero, return
    true; otherwise, return false
es5id: 11.8.2_A4.9
description: x and y are number primitives
---*/

//CHECK#1
if ((1 > 1.1) !== false) {
  throw new Test262Error('#1: (1 > 1.1) === false');
}

//CHECK#2
if ((1.1 > 1) !== true) {
  throw new Test262Error('#2: (1.1 > 1) === true');
}

//CHECK#3
if ((-1 > -1.1) !== true) {
  throw new Test262Error('#3: (-1 > -1.1) === true');
}

//CHECK#4
if ((-1.1 > -1) !== false) {
  throw new Test262Error('#4: (-1.1 > -1) === false');
}

//CHECK#5
if ((0.1 > 0) !== true) {
  throw new Test262Error('#5: (0.1 > 0) === true');
}

//CHECK#6
if ((0 > -0.1) !== true) {
  throw new Test262Error('#6: (0 > -0.1) === true');
}

//CHECK#7
if ((Number.MAX_VALUE > Number.MAX_VALUE/2) !== true) {
  throw new Test262Error('#7: (Number.MAX_VALUE > Number.MAX_VALUE/2) === true');
}

//CHECK#8
if ((Number.MIN_VALUE*2 > Number.MIN_VALUE) !== true) {
  throw new Test262Error('#8: (Number.MIN_VALUE*2 > Number.MIN_VALUE) === true');
}
