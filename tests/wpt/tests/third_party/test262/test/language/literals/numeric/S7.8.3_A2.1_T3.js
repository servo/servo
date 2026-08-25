// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: .DecimalDigits"
es5id: 7.8.3_A2.1_T3
description: Use .DecimalDigits that have at the end zeros
---*/

//CHECK#0
if (.00 !== 0.0) {
  throw new Test262Error('#0: .0 === 0.0');
}

//CHECK#1
if (.10 !== 0.1) {
  throw new Test262Error('#1: .1 === 0.1');
}

//CHECK#2
if (.20 !== 0.2) {
  throw new Test262Error('#2: .2 === 0.2');
}

//CHECK#3
if (.30 !== 0.3) {
  throw new Test262Error('#3: .3 === 0.3');
}

//CHECK#4
if (.40 !== 0.4) {
  throw new Test262Error('#4: .4 === 0.4');
}

//CHECK#5
if (.50 !== 0.5) {
  throw new Test262Error('#5: .5 === 0.5');
}

//CHECK#6
if (.60 !== 0.6) {
  throw new Test262Error('#6: .6 === 0.6');
}

//CHECK#7
if (.70 !== 0.7) {
  throw new Test262Error('#7: .7 === 0.7');
}

//CHECK#8
if (.80 !== 0.8) {
  throw new Test262Error('#8: .8 === 0.8');
}

//CHECK#9
if (.90 !== 0.9) {
  throw new Test262Error('#9: .9 === 0.9');
}
