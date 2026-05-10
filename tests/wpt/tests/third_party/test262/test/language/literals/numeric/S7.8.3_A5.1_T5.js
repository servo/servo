// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: HexIntegerLiteral"
es5id: 7.8.3_A5.1_T5
description: "HexIntegerLiteral :: 0x0 Digits"
---*/

//CHECK#0
if (0x00 !== 0) {
  throw new Test262Error('#0: 0x00 === 0');
}

//CHECK#1
if (0x01 !== 1) {
  throw new Test262Error('#1: 0x01 === 1');
}

//CHECK#2
if (0x010 !== 16) {
  throw new Test262Error('#2: 0x010 === 16');
}

//CHECK3
if (0x0100 !== 256) {
  throw new Test262Error('3: 0x0100 === 256');
}

//CHECK#4
if (0x01000 !== 4096) {
  throw new Test262Error('#4: 0x01000 === 4096');
}

//CHECK#5
if (0x010000 !== 65536) {
  throw new Test262Error('#5: 0x010000 === 65536');
}

//CHECK#6
if (0x0100000 !== 1048576) {
  throw new Test262Error('#6: 0x0100000 === 1048576');
}

//CHECK#7
if (0x01000000 !== 16777216) {
  throw new Test262Error('#7: 0x01000000 === 16777216');
}

//CHECK#8
if (0x010000000 !== 268435456) {
  throw new Test262Error('#8: 0x010000000 === 268435456');
}
