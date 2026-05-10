// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase()
es5id: 15.5.4.16_A1_T3
description: Checking by using eval
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (eval("\"BJ\"").toLowerCase() !== "bj") {
  throw new Test262Error('#1: eval("\\"BJ\\"").toLowerCase() === "bj". Actual: ' + eval("\"BJ\"").toLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
