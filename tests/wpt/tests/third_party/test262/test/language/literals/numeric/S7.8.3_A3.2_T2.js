// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral. DecimalDigits"
es5id: 7.8.3_A3.2_T2
description: After DecimalIntegerLiteral. used ZeroDigits
---*/

//CHECK#0
if (0.00 !== 0) {
  throw new Test262Error('#0: 0.00 === 0');
}

//CHECK#1
if (1.00 !== 1) {
  throw new Test262Error('#1: 1.00 === 1');
}

//CHECK#2
if (2.00 !== 2) {
  throw new Test262Error('#2: 2.00 === 2');
}

//CHECK#3
if (3.00 !== 3) {
  throw new Test262Error('#3: 3.00 === 3');
}

//CHECK#4
if (4.00 !== 4) {
  throw new Test262Error('#4: 4.00 === 4');
}

//CHECK#5
if (5.00 !== 5) {
  throw new Test262Error('#5: 5.00 === 5');
}

//CHECK#6
if (6.00 !== 6) {
  throw new Test262Error('#6: 6.00 === 6');
}

//CHECK#7
if (7.00 !== 7) {
  throw new Test262Error('#7: 7.00 === 7');
}

//CHECK#8
if (8.00 !== 8) {
  throw new Test262Error('#8: 8.00 === 8');
}

//CHECK#9
if (9.00 !== 9) {
  throw new Test262Error('#9: 9.00 === 9');
}
