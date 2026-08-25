// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator +x returns ToNumber(x)
es5id: 11.4.6_A3_T3
description: Type(x) is string primitive or String object
---*/

//CHECK#1
if (+"1" !== 1) {
  throw new Test262Error('#1: +"1" === 1. Actual: ' + (+"1"));
}

//CHECK#2
if (+new Number("-1") !== -1) {
  throw new Test262Error('#2: +new String("-1") === -1. Actual: ' + (+new String("-1")));
}

//CHECK#3
if (isNaN(+"x") !== true) {
  throw new Test262Error('#3: +"x" === Not-a-Number. Actual: ' + (+"x"));
}

//CHECK#4
if (isNaN(+"INFINITY") !== true) {
  throw new Test262Error('#4: +"INFINITY" === Not-a-Number. Actual: ' + (+"INFINITY"));
}

//CHECK#5
if (isNaN(+"infinity") !== true) {
  throw new Test262Error('#5: +"infinity" === Not-a-Number. Actual: ' + (+"infinity"));
}
