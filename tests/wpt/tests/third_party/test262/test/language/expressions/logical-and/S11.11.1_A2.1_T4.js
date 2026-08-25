// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x && y uses GetValue
es5id: 11.11.1_A2.1_T4
description: If ToBoolean(x) is false and GetBase(y) is null, return false
---*/

//CHECK#1
if ((false && x) !== false) {
  throw new Test262Error('#1: (false && x) === false');
}
