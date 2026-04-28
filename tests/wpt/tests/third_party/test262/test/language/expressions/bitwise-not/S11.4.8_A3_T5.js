// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator ~x returns ~ToInt32(x)
es5id: 11.4.8_A3_T5
description: Type(x) is Object object or Function object
---*/

//CHECK#1
if (~({}) !== -1) {
  throw new Test262Error('#1: ~({}) === -1. Actual: ' + (~({})));
}

//CHECK#2  
if (~(function(){return 1}) !== -1) {
  throw new Test262Error('#2: ~(function(){return 1}) === -1. Actual: ' + (~(function(){return 1})));
}
