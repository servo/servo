// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator -x returns -ToNumber(x)
es5id: 11.4.7_A3_T5
description: Type(x) is Object object or Function object
---*/

//CHECK#1
if (isNaN(-{}) !== true) {
  throw new Test262Error('#1: -{} === Not-a-Number. Actual: ' + (-{}));
}

//CHECK#2  
if (isNaN(-function(){return 1}) !== true) {
  throw new Test262Error('#2: -function(){return 1} === Not-a-Number. Actual: ' + (-function(){return 1}));
}
