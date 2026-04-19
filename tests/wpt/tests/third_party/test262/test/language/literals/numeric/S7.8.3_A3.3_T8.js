// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral. ExponentPart"
es5id: 7.8.3_A3.3_T8
description: "ExponentPart :: E 0"
---*/

//CHECK#0
if (0.E0 !== 0) {
  throw new Test262Error('#0: 0.E0 === 0');
}

//CHECK#1
if (1.E0 !== 1) {
  throw new Test262Error('#1: 1.E0 === 1');
}

//CHECK#2
if (2.E0 !== 2) {
  throw new Test262Error('#2: 2.E0 === 2');
}

//CHECK#3
if (3.E0 !== 3) {
  throw new Test262Error('#3: 3.E0 === 3');
}

//CHECK#4
if (4.E0 !== 4) {
  throw new Test262Error('#4: 4.E0 === 4');
}

//CHECK#5
if (5.E0 !== 5) {
  throw new Test262Error('#5: 5.E0 === 5');
}

//CHECK#6
if (6.E0 !== 6) {
  throw new Test262Error('#6: 6.E0 === 6');
}

//CHECK#7
if (7.E0 !== 7) {
  throw new Test262Error('#7: 7.E0 === 7');
}

//CHECK#8
if (8.E0 !== 8) {
  throw new Test262Error('#8: 8.E0 === 8');
}

//CHECK#9
if (9.E0 !== 9) {
  throw new Test262Error('#9: 9.E0 === 9');
}
