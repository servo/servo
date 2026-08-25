// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Literal :: NullLiteral"
es5id: 7.8.1_A1_T2
description: Check RegExp("0").exec("1") === null
---*/

//CHECK#1
if (RegExp("0").exec("1") !== null) {
  throw new Test262Error('#1: RegExp("0").exec("1") === null');
}
