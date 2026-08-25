// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Literal :: BooleanLiteral"
es5id: 7.8.2_A1_T1
description: "BooleanLiteral :: true"
---*/

//CHECK#1
if (Boolean(true) !== true) {
  throw new Test262Error('#1: Boolean(true) === true. Actual: Boolean(true) === ' + (Boolean(true)));
}
