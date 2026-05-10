// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the indexOf method is 1
es5id: 15.5.4.7_A11
description: Checking String.prototype.indexOf.length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.indexOf.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.indexOf.hasOwnProperty("length") return true. Actual: ' + String.prototype.indexOf.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.indexOf.length !== 1) {
  throw new Test262Error('#2: String.prototype.indexOf.length === 1. Actual: ' + String.prototype.indexOf.length);
}
//
//////////////////////////////////////////////////////////////////////////////
