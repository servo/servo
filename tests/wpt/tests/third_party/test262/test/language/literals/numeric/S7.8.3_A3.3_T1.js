// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral. ExponentPart"
es5id: 7.8.3_A3.3_T1
description: "ExponentPart :: e DecimalDigits"
---*/

//CHECK#0
if (0.e1 !== 0) {
  throw new Test262Error('#0: 0.e1 === 0');
}

//CHECK#1
if (1.e1 !== 10) {
  throw new Test262Error('#1: 1.e1 === 10');
}

//CHECK#2
if (2.e1 !== 20) {
  throw new Test262Error('#2: 2.e1 === 20');
}

//CHECK#3
if (3.e1 !== 30) {
  throw new Test262Error('#3: 3.e1 === 30');
}

//CHECK#4
if (4.e1 !== 40) {
  throw new Test262Error('#4: 4.e1 === 40');
}

//CHECK#5
if (5.e1 !== 50) {
  throw new Test262Error('#5: 5.e1 === 50');
}

//CHECK#6
if (6.e1 !== 60) {
  throw new Test262Error('#6: 6.e1 === 60');
}

//CHECK#7
if (7.e1 !== 70) {
  throw new Test262Error('#7: 7.e1 === 70');
}

//CHECK#8
if (8.e1 !== 80) {
  throw new Test262Error('#8: 8.e1 === 80');
}

//CHECK#9
if (9.e1 !== 90) {
  throw new Test262Error('#9: 9.e1 === 90');
}
