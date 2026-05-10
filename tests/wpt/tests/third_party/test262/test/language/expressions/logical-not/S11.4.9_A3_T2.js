// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator !x returns !ToBoolean(x)
es5id: 11.4.9_A3_T2
description: Type(x) is number primitive or Number object
---*/

//CHECK#1
if (!0.1 !== false) {
  throw new Test262Error('#1: !0.1 === false');
}

//CHECK#2
if (!new Number(-0.1) !== false) {
  throw new Test262Error('#2: !new Number(-0.1) === false');
}

//CHECK#3
if (!NaN !== true) {
  throw new Test262Error('#3: !NaN === true');
}

//CHECK#4
if (!new Number(NaN) !== false) {
  throw new Test262Error('#4: !new Number(NaN) === false');
}

//CHECK#5
if (!0 !== true) {
  throw new Test262Error('#5: !0 === true');
}

//CHECK#6
if (!new Number(0) !== false) {
  throw new Test262Error('#6: !new Number(0) === false');
}

//CHECK#7
if (!Infinity !== false) {
  throw new Test262Error('#7: !Infinity === false');
}
