// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charCodeAt(pos)
es5id: 15.5.4.5_A1_T9
description: >
    Call charCodeAt() function with function(){}() argument of string
    object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToInteger(undefined) evaluates to 0 charCodeAt() evaluates to charCodeAt(0)
if (new String(42).charCodeAt(function() {}()) !== 0x34) {
  throw new Test262Error('#1: new String(42).charCodeAt(function(){}()) === 0x34. Actual: new String(42).charCodeAt(function(){}()) ===' + new String(42).charCodeAt(function() {}()));
}
//
//////////////////////////////////////////////////////////////////////////////
