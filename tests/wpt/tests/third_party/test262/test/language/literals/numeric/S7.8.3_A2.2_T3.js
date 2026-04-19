// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: .DecimalDigits ExponentPart"
es5id: 7.8.3_A2.2_T3
description: "ExponentPart :: e +DecimalDigits"
---*/

//CHECK#0
if (.0e-1 !== 0) {
  throw new Test262Error('#0: .0e-1 === 0');
}

//CHECK#1
if (.1e-1 !== 0.01) {
  throw new Test262Error('#1: .1e-1 === 0.01');
}

//CHECK#2
if (.2e-1 !== 0.02) {
  throw new Test262Error('#2: .2e-1 === 0.02');
}

//CHECK#3
if (.3e-1 !== 0.03) {
  throw new Test262Error('#3: .3e-1 === 0.03');
}

//CHECK#4
if (.4e-1 !== 0.04) {
  throw new Test262Error('#4: .4e-1 === 0.04');
}

//CHECK#5
if (.5e-1 !== 0.05) {
  throw new Test262Error('#5: .5e-1 === 0.05');
}

//CHECK#6
if (.6e-1 !== 0.06) {
  throw new Test262Error('#6: .6e-1 === 0.06');
}

//CHECK#7
if (.7e-1 !== 0.07) {
  throw new Test262Error('#7: .7e-1 === 0.07');
}

//CHECK#8
if (.8e-1 !== 0.08) {
  throw new Test262Error('#8: .8e-1 === 0.08');
}

//CHECK#9
if (.9e-1 !== 0.09) {
  throw new Test262Error('#9: .9e-1 === 0.09');
}
