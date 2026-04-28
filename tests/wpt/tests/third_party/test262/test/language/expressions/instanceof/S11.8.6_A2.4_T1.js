// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.8.6_A2.4_T1
description: Checking with "="
---*/

//CHECK#1 
var OBJECT = 0;
if ((OBJECT = Object, {}) instanceof OBJECT !== true) {
  throw new Test262Error('#1: var OBJECT = 0; (OBJECT = Object, {}) instanceof OBJECT === true');
}

//CHECK#2
var object = {}; 
if (object instanceof (object = 0, Object) !== true) {
  throw new Test262Error('#2: var object = {};  object instanceof (object = 0, Object) === true');
}
