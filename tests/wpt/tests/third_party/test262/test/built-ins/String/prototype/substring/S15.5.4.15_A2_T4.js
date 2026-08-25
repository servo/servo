// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.substring (start, end) returns a string value(not object)
es5id: 15.5.4.15_A2_T4
description: start is Infinity, end is NaN
---*/

var __string = new String("this is a string object");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__string.substring(Infinity, NaN) !== "this is a string object") {
  throw new Test262Error('#1: __string = new String("this is a string object"); __string.substring(Infinity, NaN) === "this is a string object". Actual: ' + __string.substring(Infinity, NaN));
}
//
//////////////////////////////////////////////////////////////////////////////
