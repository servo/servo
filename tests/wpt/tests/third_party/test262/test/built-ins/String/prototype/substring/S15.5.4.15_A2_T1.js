// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.substring (start, end) returns a string value(not object)
es5id: 15.5.4.15_A2_T1
description: Checking type of substring()
---*/

var __string = new String("this is a string object");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __string.substring() !== "string") {
  throw new Test262Error('#1: __string = new String("this is a string object"); typeof __string.substring() === "string". Actual: ' + typeof __string.substring());
}
//
//////////////////////////////////////////////////////////////////////////////
