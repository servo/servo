// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral. DecimalDigigts ExponentPart"
es5id: 7.8.3_A3.4_T7
description: "ExponentPart :: e 0"
---*/

//CHECK#0
if (0.0e0 !== 0.0) {
  throw new Test262Error('#0: 0.0e0 === 0.0');
}

//CHECK#1
if (1.1e0 !== 1.1) {
  throw new Test262Error('#1: 1.1e0 === 1.1');
}

//CHECK#2
if (2.2e0 !== 2.2) {
  throw new Test262Error('#2: 2.2e0 === 2.2');
}

//CHECK#3
if (3.3e0 !== 3.3) {
  throw new Test262Error('#3: 3.3e0 === 3.3');
}

//CHECK#4
if (4.4e0 !== 4.4) {
  throw new Test262Error('#4: 4.4e0 === 4.4');
}

//CHECK#5
if (5.5e0 !== 5.5) {
  throw new Test262Error('#5: 5.5e0 === 5.5');
}

//CHECK#6
if (6.6e0 !== 6.6) {
  throw new Test262Error('#6: 6.e0 === 6.6');
}

//CHECK#7
if (7.7e0 !== 7.7) {
  throw new Test262Error('#7: 7.7e0 === 7.7');
}

//CHECK#8
if (8.8e0 !== 8.8) {
  throw new Test262Error('#8: 8.8e0 === 8.8');
}

//CHECK#9
if (9.9e0 !== 9.9) {
  throw new Test262Error('#9: 9.9e0 === 9.9');
}
