// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x || y uses GetValue
es5id: 11.11.2_A2.1_T4
description: If ToBoolean(x) is true and GetBase(y) is null, return true
---*/

//CHECK#1
if ((true || x) !== true) {
  throw new Test262Error('#1: (true || x) === true');
}
