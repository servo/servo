// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end) returns a string value(not object)
es5id: 15.5.4.13_A2_T4
description: start is Infinity, end is NaN
---*/

var __string = new String("this is a string object");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__string.slice(Infinity, NaN) !== "") {
  throw new Test262Error('#1: __string = new String("this is a string object"); __string.slice(Infinity, NaN) === "". Actual: ' + __string.slice(Infinity, NaN));
}
//
//////////////////////////////////////////////////////////////////////////////
