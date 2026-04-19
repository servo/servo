// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: HexIntegerLiteral"
es5id: 7.8.3_A5.1_T7
description: "HexIntegerLiteral :: 0x one of a, b, c, d, e, f"
---*/

//CHECK#a
if (0xa !== 10) {
  throw new Test262Error('#a: 0xa === 10');
}

//CHECK#b
if (0xb !== 11) {
  throw new Test262Error('#b: 0xb === 11');
}

//CHECK#c
if (0xc !== 12) {
  throw new Test262Error('#c: 0xc === 12');
}

//CHECK#d
if (0xd !== 13) {
  throw new Test262Error('#d: 0xd === 13');
}

//CHECK#e
if (0xe !== 14) {
  throw new Test262Error('#e: 0xe === 14');
}

//CHECK#f
if (0xf !== 15) {
  throw new Test262Error('#f: 0xf === 15');
}
