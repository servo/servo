// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral ExponentPart"
es5id: 7.8.3_A1.2_T2
description: "ExponentPart :: E DecimalDigits"
---*/

//CHECK#0
if (0E1 !== 0) {
  throw new Test262Error('#0: 0E1 === 0');
}

//CHECK#1
if (1E1 !== 10) {
  throw new Test262Error('#1: 1E1 === 1');
}

//CHECK#2
if (2E1 !== 20) {
  throw new Test262Error('#2: 2E1 === 20');
}

//CHECK#3
if (3E1 !== 30) {
  throw new Test262Error('#3: 3E1 === 30');
}

//CHECK#4
if (4E1 !== 40) {
  throw new Test262Error('#4: 4E1 === 40');
}

//CHECK#5
if (5E1 !== 50) {
  throw new Test262Error('#5: 5E1 === 50');
}

//CHECK#6
if (6E1 !== 60) {
  throw new Test262Error('#6: 6E1 === 60');
}

//CHECK#7
if (7E1 !== 70) {
  throw new Test262Error('#7: 7E1 === 70');
}

//CHECK#8
if (8E1 !== 80) {
  throw new Test262Error('#8: 8E1 === 80');
}

//CHECK#9
if (9E1 !== 90) {
  throw new Test262Error('#9: 9E1 === 90');
}
