// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charCodeAt(pos)
es5id: 15.5.4.5_A1_T4
description: Call charCodeAt() function without argument of string object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since Number() evaluates to 0 charCodeAt() evaluates to charCodeAt(0)
if ("smart".charCodeAt() !== 0x73) {
  throw new Test262Error('#1: "smart".charCodeAt() === 0x73. Actual: "smart".charCodeAt() ===' + ("smart".charCodeAt()));
}
//
//////////////////////////////////////////////////////////////////////////////
