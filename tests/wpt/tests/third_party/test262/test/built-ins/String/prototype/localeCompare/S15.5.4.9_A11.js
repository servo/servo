// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the localeCompare method is 1
es5id: 15.5.4.9_A11
description: Checking String.prototype.localeCompare.length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.localeCompare.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.localeCompare.hasOwnProperty("length") return true. Actual: ' + String.prototype.localeCompare.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.localeCompare.length !== 1) {
  throw new Test262Error('#2: String.prototype.localeCompare.length === 1. Actual: ' + String.prototype.localeCompare.length);
}
//
//////////////////////////////////////////////////////////////////////////////
