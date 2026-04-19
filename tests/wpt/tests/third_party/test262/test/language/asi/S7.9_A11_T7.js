// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check If Statement for automatic semicolon insertion
es5id: 7.9_A11_T7
description: Use if (false) x = 1; \n else x=-1 and check x
---*/

//CHECK#1
var x = 0;
if (false) x = 1;
else x = -1
if (x !== -1) {
  throw new Test262Error('#1: Check If Statement for automatic semicolon insertion');
}
