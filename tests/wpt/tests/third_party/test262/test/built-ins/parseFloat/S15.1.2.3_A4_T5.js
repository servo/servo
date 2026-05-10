// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Compute the longest prefix of Result(2), which might be Result(2) itself,
    which satisfies the syntax of a StrDecimalLiteral
esid: sec-parsefloat-string
description: Checking DecimalDigits . DecimalDigits_opt ExponentPart_opt
---*/

//CHECK#1
if (parseFloat("-11.string") !== -11) {
  throw new Test262Error('#1: parseFloat("-11.string") === -11. Actual: ' + (parseFloat("-11.string")));
}

//CHECK#2
if (parseFloat("01.string") !== 1) {
  throw new Test262Error('#2: parseFloat("01.string") === 1. Actual: ' + (parseFloat("01.string")));
}

//CHECK#3
if (parseFloat("+11.1string") !== 11.1) {
  throw new Test262Error('#3: parseFloat("+11.1string") === 11.1. Actual: ' + (parseFloat("+11.1string")));
}

//CHECK#4
if (parseFloat("01.1string") !== 1.1) {
  throw new Test262Error('#4: parseFloat("01.1string") === 1.1. Actual: ' + (parseFloat("01.1string")));
}

//CHECK#5
if (parseFloat("-11.e-1string") !== -1.1) {
  throw new Test262Error('#5: parseFloat("-11.e-1string") === -1.1. Actual: ' + (parseFloat("-11.e-1string")));
}

//CHECK#6
if (parseFloat("01.e1string") !== 10) {
  throw new Test262Error('#6: parseFloat("01.e1string") === 10. Actual: ' + (parseFloat("01.e1string")));
}

//CHECK#7
if (parseFloat("+11.22e-1string") !== 1.122) {
  throw new Test262Error('#7: parseFloat("+11.22e-1string") === 1.122. Actual: ' + (parseFloat("+11.22e-1string")));
}

//CHECK#8
if (parseFloat("01.01e1string") !== 10.1) {
  throw new Test262Error('#8: parseFloat("01.01e1string") === 10.1. Actual: ' + (parseFloat("01.01e1string")));
}

//CHECK#9
if (parseFloat("001.string") !== 1) {
  throw new Test262Error('#9: parseFloat("001.string") === 1. Actual: ' + (parseFloat("001.string")));
}

//CHECK#10
if (parseFloat("010.string") !== 10) {
  throw new Test262Error('#10: parseFloat("010.string") === 10. Actual: ' + (parseFloat("010.string")));
}
