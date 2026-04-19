// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral. ExponentPart"
es5id: 7.8.3_A3.3_T7
description: "ExponentPart :: e 0"
---*/

//CHECK#0
if (0.e0 !== 0) {
  throw new Test262Error('#0: 0.e0 === 0');
}

//CHECK#1
if (1.e0 !== 1) {
  throw new Test262Error('#1: 1.e0 === 1');
}

//CHECK#2
if (2.e0 !== 2) {
  throw new Test262Error('#2: 2.e0 === 2');
}

//CHECK#3
if (3.e0 !== 3) {
  throw new Test262Error('#3: 3.e0 === 3');
}

//CHECK#4
if (4.e0 !== 4) {
  throw new Test262Error('#4: 4.e0 === 4');
}

//CHECK#5
if (5.e0 !== 5) {
  throw new Test262Error('#5: 5.e0 === 5');
}

//CHECK#6
if (6.e0 !== 6) {
  throw new Test262Error('#6: 6.e0 === 6');
}

//CHECK#7
if (7.e0 !== 7) {
  throw new Test262Error('#7: 7.e0 === 7');
}

//CHECK#8
if (8.e0 !== 8) {
  throw new Test262Error('#8: 8.e0 === 8');
}

//CHECK#9
if (9.e0 !== 9) {
  throw new Test262Error('#9: 9.e0 === 9');
}
