// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the toLocaleUpperCase method is 0
es5id: 15.5.4.19_A11
description: Checking String.prototype.toLocaleUpperCase.length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!(String.prototype.toLocaleUpperCase.hasOwnProperty("length"))) {
  throw new Test262Error('#1: String.prototype.toLocaleUpperCase.hasOwnProperty("length") return true. Actual: ' + String.prototype.toLocaleUpperCase.hasOwnProperty("length"));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (String.prototype.toLocaleUpperCase.length !== 0) {
  throw new Test262Error('#2: String.prototype.toLocaleUpperCase.length === 0. Actual: ' + String.prototype.toLocaleUpperCase.length);
}
//
//////////////////////////////////////////////////////////////////////////////
