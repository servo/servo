// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: HexIntegerLiteral"
es5id: 7.8.3_A5.1_T1
description: "HexIntegerLiteral :: 0x Digit"
---*/

//CHECK#0
if (0x0 !== 0) {
  throw new Test262Error('#0: 0x0 === 0');
}

//CHECK#1
if (0x1 !== 1) {
  throw new Test262Error('#1: 0x1 === 1');
}

//CHECK#2
if (0x2 !== 2) {
  throw new Test262Error('#2: 0x2 === 2');
}

//CHECK#3
if (0x3 !== 3) {
  throw new Test262Error('#3: 0x3 === 3');
}

//CHECK#4
if (0x4 !== 4) {
  throw new Test262Error('#4: 0x4 === 4');
}

//CHECK#5
if (0x5 !== 5) {
  throw new Test262Error('#5: 0x5 === 5');
}

//CHECK#6
if (0x6 !== 6) {
  throw new Test262Error('#6: 0x6 === 6');
}

//CHECK#7
if (0x7 !== 7) {
  throw new Test262Error('#7: 0x7 === 7');
}

//CHECK#8
if (0x8 !== 8) {
  throw new Test262Error('#8: 0x8 === 8');
}

//CHECK#9
if (0x9 !== 9) {
  throw new Test262Error('#9: 0x9 === 9');
}

//CHECK#A
if (0xA !== 10) {
  throw new Test262Error('#A: 0xA === 10');
}

//CHECK#B
if (0xB !== 11) {
  throw new Test262Error('#B: 0xB === 11');
}

//CHECK#C
if (0xC !== 12) {
  throw new Test262Error('#C: 0xC === 12');
}

//CHECK#D
if (0xD !== 13) {
  throw new Test262Error('#D: 0xD === 13');
}

//CHECK#E
if (0xE !== 14) {
  throw new Test262Error('#E: 0xE === 14');
}

//CHECK#F
if (0xF !== 15) {
  throw new Test262Error('#F: 0xF === 15');
}
