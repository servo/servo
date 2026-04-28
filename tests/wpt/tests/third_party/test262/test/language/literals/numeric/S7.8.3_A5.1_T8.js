// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: HexIntegerLiteral"
es5id: 7.8.3_A5.1_T8
description: "HexIntegerLiteral :: 0X one of a, b, c, d, e, f"
---*/

//CHECK#a
if (0Xa !== 10) {
  throw new Test262Error('#a: 0Xa === 10');
}

//CHECK#b
if (0Xb !== 11) {
  throw new Test262Error('#b: 0Xb === 11');
}

//CHECK#c
if (0Xc !== 12) {
  throw new Test262Error('#c: 0Xc === 12');
}

//CHECK#d
if (0Xd !== 13) {
  throw new Test262Error('#d: 0Xd === 13');
}

//CHECK#e
if (0Xe !== 14) {
  throw new Test262Error('#e: 0Xe === 14');
}

//CHECK#f
if (0Xf !== 15) {
  throw new Test262Error('#f: 0Xf === 15');
}
