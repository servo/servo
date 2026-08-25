// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the concat method is 1
es5id: 15.5.4.6_A11
description: Checking String.prototype.concat.length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.concat.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.concat.hasOwnProperty("length") return true. Actual: ' + String.prototype.concat.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.concat.length !== 1) {
  throw new Test262Error('#2: String.prototype.concat.length === 1. Actual: ' + String.prototype.concat.length);
}
//
//////////////////////////////////////////////////////////////////////////////
