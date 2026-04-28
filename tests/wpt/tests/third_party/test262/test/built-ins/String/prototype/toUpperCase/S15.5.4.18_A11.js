// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the toUpperCase method is 0
es5id: 15.5.4.18_A11
description: Checking String.prototype.toUpperCase.length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.toUpperCase.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.toUpperCase.hasOwnProperty("length") return true. Actual: ' + String.prototype.toUpperCase.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.toUpperCase.length !== 0) {
  throw new Test262Error('#2: String.prototype.toUpperCase.length === 0. Actual: ' + String.prototype.toUpperCase.length);
}
//
//////////////////////////////////////////////////////////////////////////////
