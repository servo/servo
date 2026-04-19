// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator -x returns -ToNumber(x)
es5id: 11.4.7_A3_T4
description: Type(x) is undefined or null
---*/

//CHECK#1
if (isNaN(-void 0) !== true) {
  throw new Test262Error('#1: +void 0 === Not-a-Number. Actual: ' + (+void 0));
}

//CHECK#2
if (-null !== 0) {
  throw new Test262Error('#2: +null === 0. Actual: ' + (+null));
}
