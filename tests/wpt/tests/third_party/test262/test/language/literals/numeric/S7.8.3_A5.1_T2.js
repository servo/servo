// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: HexIntegerLiteral"
es5id: 7.8.3_A5.1_T2
description: "HexIntegerLiteral :: 0X Digit"
---*/

//CHECK#0
if (0X0 !== 0) {
  throw new Test262Error('#0: 0X0 === 0');
}

//CHECK#1
if (0X1 !== 1) {
  throw new Test262Error('#1: 0X1 === 1');
}

//CHECK#2
if (0X2 !== 2) {
  throw new Test262Error('#2: 0X2 === 2');
}

//CHECK#3
if (0X3 !== 3) {
  throw new Test262Error('#3: 0X3 === 3');
}

//CHECK#4
if (0X4 !== 4) {
  throw new Test262Error('#4: 0X4 === 4');
}

//CHECK#5
if (0X5 !== 5) {
  throw new Test262Error('#5: 0X5 === 5');
}

//CHECK#6
if (0X6 !== 6) {
  throw new Test262Error('#6: 0X6 === 6');
}

//CHECK#7
if (0X7 !== 7) {
  throw new Test262Error('#7: 0X7 === 7');
}

//CHECK#8
if (0X8 !== 8) {
  throw new Test262Error('#8: 0X8 === 8');
}

//CHECK#9
if (0X9 !== 9) {
  throw new Test262Error('#9: 0X9 === 9');
}

//CHECK#A
if (0XA !== 10) {
  throw new Test262Error('#A: 0XA === 10');
}

//CHECK#B
if (0XB !== 11) {
  throw new Test262Error('#B: 0XB === 11');
}

//CHECK#C
if (0XC !== 12) {
  throw new Test262Error('#C: 0XC === 12');
}

//CHECK#D
if (0XD !== 13) {
  throw new Test262Error('#D: 0XD === 13');
}

//CHECK#E
if (0XE !== 14) {
  throw new Test262Error('#E: 0XE === 14');
}

//CHECK#F
if (0XF !== 15) {
  throw new Test262Error('#F: 0XF === 15');
}
