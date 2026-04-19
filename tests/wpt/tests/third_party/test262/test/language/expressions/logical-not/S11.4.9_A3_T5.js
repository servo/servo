// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator !x returns !ToBoolean(x)
es5id: 11.4.9_A3_T5
description: Type(x) is Object object or Function object
---*/

//CHECK#1
if ((!{}) !== false) {
  throw new Test262Error('#1: !({}) === false');
}

//CHECK#2  
if (!(function(){return 1}) !== false) {
  throw new Test262Error('#2: !(function(){return 1}) === false');
}
