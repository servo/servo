// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charAt(pos)
es5id: 15.5.4.4_A1_T4
description: Call charAt() function without argument of string object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToInteger() evaluates to 0 charAt() evaluates to charAt(0)
if ("lego".charAt() !== "l") {
  throw new Test262Error('#1: "lego".charAt() === "l". Actual: "lego".charAt() ===' + ("lego".charAt()));
}
//
//////////////////////////////////////////////////////////////////////////////
