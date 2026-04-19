// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "ExponentPart :: ExponentIndicator ( /+/-) 0 DecimalDigits is allowed"
es5id: 7.8.3_A4.2_T1
description: "ExponentIndicator :: e"
---*/

//CHECK#0
if (0e01 !== 0) {
  throw new Test262Error('#0: 0e01 === 0');
}

//CHECK#1
if (1e01 !== 10) {
  throw new Test262Error('#1: 1e01 === 10');
}

//CHECK#2
if (2e01 !== 20) {
  throw new Test262Error('#2: 2e01 === 20');
}

//CHECK#3
if (3e01 !== 30) {
  throw new Test262Error('#3: 3e01 === 30');
}

//CHECK#4
if (4e01 !== 40) {
  throw new Test262Error('#4: 4e01 === 40');
}

//CHECK#5
if (5e01 !== 50) {
  throw new Test262Error('#5: 5e01 === 50');
}

//CHECK#6
if (6e01 !== 60) {
  throw new Test262Error('#6: 6e01 === 60');
}

//CHECK#7
if (7e01 !== 70) {
  throw new Test262Error('#7: 7e01 === 70');
}

//CHECK#8
if (8e01 !== 80) {
  throw new Test262Error('#8: 8e01 === 80');
}

//CHECK#9
if (9e01 !== 90) {
  throw new Test262Error('#9: 9e01 === 90');
}
