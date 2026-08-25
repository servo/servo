// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charAt(pos)
es5id: 15.5.4.4_A1_T9
description: >
    Call charAt() function with function(){}() argument of string
    object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToInteger(undefined) evaluates to 0 charAt() evaluates to charAt(0)
if (new String(42).charAt(function() {}()) !== "4") {
  throw new Test262Error('#1: new String(42).charAt(function(){}()) === "4". Actual: new String(42).charAt(function(){}()) ===' + new String(42).charAt(function() {}()));
}
//
//////////////////////////////////////////////////////////////////////////////
