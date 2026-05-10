// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral ExponentPart"
es5id: 7.8.3_A1.2_T5
description: "ExponentPart :: e +DecimalDigits"
---*/

//CHECK#0
if (0e+1 !== 0) {
  throw new Test262Error('#0: 0e+1 === 0');
}

//CHe+CK#1
if (1e+1 !== 10) {
  throw new Test262Error('#1: 1e+1 === 10');
}

//CHe+CK#2
if (2e+1 !== 20) {
  throw new Test262Error('#2: 2e+1 === 20');
}

//CHe+CK#3
if (3e+1 !== 30) {
  throw new Test262Error('#3: 3e+1 === 30');
}

//CHe+CK#4
if (4e+1 !== 40) {
  throw new Test262Error('#4: 4e+1 === 40');
}

//CHe+CK#5
if (5e+1 !== 50) {
  throw new Test262Error('#5: 5e+1 === 50');
}

//CHe+CK#6
if (6e+1 !== 60) {
  throw new Test262Error('#6: 6e+1 === 60');
}

//CHe+CK#7
if (7e+1 !== 70) {
  throw new Test262Error('#7: 7e+1 === 70');
}

//CHe+CK#8
if (8e+1 !== 80) {
  throw new Test262Error('#8: 8e+1 === 80');
}

//CHe+CK#9
if (9e+1 !== 90) {
  throw new Test262Error('#9: 9e+1 === 90');
}
