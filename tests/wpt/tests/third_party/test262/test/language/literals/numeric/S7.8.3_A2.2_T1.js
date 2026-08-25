// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: .DecimalDigits ExponentPart"
es5id: 7.8.3_A2.2_T1
description: "ExponentPart :: e DecimalDigits"
---*/

//CHECK#0
if (.0e1 !== 0) {
  throw new Test262Error('#0: .0e1 === 0');
}

//CHECK#1
if (.1e1 !== 1) {
  throw new Test262Error('#1: .1e1 === 1');
}

//CHECK#2
if (.2e1 !== 2) {
  throw new Test262Error('#2: .2e1 === 2');
}

//CHECK#3
if (.3e1 !== 3) {
  throw new Test262Error('#3: .3e1 === 3');
}

//CHECK#4
if (.4e1 !== 4) {
  throw new Test262Error('#4: .4e1 === 4');
}

//CHECK#5
if (.5e1 !== 5) {
  throw new Test262Error('#5: .5e1 === 5');
}

//CHECK#6
if (.6e1 !== 6) {
  throw new Test262Error('#6: .6e1 === 6');
}

//CHECK#7
if (.7e1 !== 7) {
  throw new Test262Error('#7: .7e1 === 7');
}

//CHECK#8
if (.8e1 !== 8) {
  throw new Test262Error('#8: .8e1 === 8');
}

//CHECK#9
if (.9e1 !== 9) {
  throw new Test262Error('#9: .9e1 === 9');
}
