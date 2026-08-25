// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: .DecimalDigits"
es5id: 7.8.3_A2.1_T2
description: Use .DecimalDigits
---*/

//CHECK#0
if (.00 !== 0.00) {
  throw new Test262Error('#0: .00 === 0.00');
}

//CHECK#1
if (.11 !== 0.11) {
  throw new Test262Error('#1: .11 === 0.11');
}

//CHECK#2
if (.22 !== 0.22) {
  throw new Test262Error('#2: .22 === 0.22');
}

//CHECK#3
if (.33 !== 0.33) {
  throw new Test262Error('#3: .33 === 0.33');
}

//CHECK#4
if (.44 !== 0.44) {
  throw new Test262Error('#4: .44 === 0.44');
}

//CHECK#5
if (.55 !== 0.55) {
  throw new Test262Error('#5: .55 === 0.55');
}

//CHECK#6
if (.66 !== 0.66) {
  throw new Test262Error('#6: .66 === 0.66');
}

//CHECK#7
if (.77 !== 0.77) {
  throw new Test262Error('#7: .77 === 0.77');
}

//CHECK#8
if (.88 !== 0.88) {
  throw new Test262Error('#8: .88 === 0.88');
}

//CHECK#9
if (.99 !== 0.99) {
  throw new Test262Error('#9: .99 === 0.99');
}
