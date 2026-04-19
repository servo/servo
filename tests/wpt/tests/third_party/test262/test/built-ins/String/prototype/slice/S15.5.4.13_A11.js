// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the slice method is 2
es5id: 15.5.4.13_A11
description: Checking String.prototype.slice.length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.slice.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.slice.hasOwnProperty("length") return true. Actual: ' + String.prototype.slice.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.slice.length !== 2) {
  throw new Test262Error('#2: String.prototype.slice.length === 2. Actual: ' + String.prototype.slice.length);
}
//
//////////////////////////////////////////////////////////////////////////////
