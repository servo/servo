// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: DecimalIntegerLiteral."
es5id: 7.8.3_A3.1_T2
description: "DecimalIntegerLiteral :: NoNZeroDigit DecimalDigigts"
---*/

//CHECK#1
if (11. !== 11) {
  throw new Test262Error('#1: 11. === 11');
}

//CHECK#2
if (22. !== 22) {
  throw new Test262Error('#2: 22. === 22');
}

//CHECK#3
if (33. !== 33) {
  throw new Test262Error('#3: 33. === 33');
}

//CHECK#4
if (44. !== 44) {
  throw new Test262Error('#4: 44. === 44');
}

//CHECK#5
if (55. !== 55) {
  throw new Test262Error('#5: 55. === 55');
}

//CHECK#6
if (66. !== 66) {
  throw new Test262Error('#6: 66. === 66');
}

//CHECK#7
if (77. !== 77) {
  throw new Test262Error('#7: 77. === 77');
}

//CHECK#8
if (88. !== 88) {
  throw new Test262Error('#8: 88. === 88');
}

//CHECK#9
if (99. !== 99) {
  throw new Test262Error('#9: 99. === 99');
}
