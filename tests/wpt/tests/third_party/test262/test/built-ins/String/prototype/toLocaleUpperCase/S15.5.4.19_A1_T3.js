// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleUpperCase()
es5id: 15.5.4.19_A1_T3
description: Checking by using eval
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (eval("\"bj\"").toLocaleUpperCase() !== "BJ") {
  throw new Test262Error('#1: eval("\\"bj\\"").toLocaleUpperCase() === "BJ". Actual: ' + eval("\"bj\"").toLocaleUpperCase());
}
//
//////////////////////////////////////////////////////////////////////////////
