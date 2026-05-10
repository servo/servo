// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toUpperCase has not prototype property
es5id: 15.5.4.18_A6
description: Checking String.prototype.toUpperCase.prototype
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String.prototype.toUpperCase.prototype !== undefined) {
  throw new Test262Error('#1: String.prototype.toUpperCase.prototype === undefined. Actual: ' + String.prototype.toUpperCase.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////
