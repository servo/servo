// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "ExponentPart :: ExponentIndicator ( /+/-) 0 DecimalDigits is allowed"
es5id: 7.8.3_A4.2_T3
description: "ExponentIndicator :: e"
---*/

//CHECK#0
if (0e-01 !== 0) {
  throw new Test262Error('#0: 0e-01 === 0');
}

//CHECK#1
if (1e-01 !== 0.1) {
  throw new Test262Error('#1: 1e-01 === 0.1');
}

//CHECK#2
if (2e-01 !== 0.2) {
  throw new Test262Error('#2: 2e-01 === 0.2');
}

//CHECK#3
if (3e-01 !== 0.3) {
  throw new Test262Error('#3: 3e-01 === 0.3');
}

//CHECK#4
if (4e-01 !== 0.4) {
  throw new Test262Error('#4: 4e-01 === 0.4');
}

//CHECK#5
if (5e-01 !== 0.5) {
  throw new Test262Error('#5: 5e-01 === 0.5');
}

//CHECK#6
if (6e-01 !== 0.6) {
  throw new Test262Error('#6: 6e-01 === 0.6');
}

//CHECK#7
if (7e-01 !== 0.7) {
  throw new Test262Error('#7: 7e-01 === 0.7');
}

//CHECK#8
if (8e-01 !== 0.8) {
  throw new Test262Error('#8: 8e-01 === 0.8');
}

//CHECK#9
if (9e-01 !== 0.9) {
  throw new Test262Error('#9: 9e-01 === 0.9');
}
