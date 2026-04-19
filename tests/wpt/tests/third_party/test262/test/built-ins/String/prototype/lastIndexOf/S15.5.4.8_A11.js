// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the lastIndexOf method is 1
es5id: 15.5.4.8_A11
description: Checking String.prototype.lastIndexOf.length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.lastIndexOf.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.lastIndexOf.hasOwnProperty("length") return true. Actual: ' + String.prototype.lastIndexOf.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.lastIndexOf.length !== 1) {
  throw new Test262Error('#2: String.prototype.lastIndexOf.length === 1. Actual: ' + String.prototype.lastIndexOf.length);
}
//
//////////////////////////////////////////////////////////////////////////////
