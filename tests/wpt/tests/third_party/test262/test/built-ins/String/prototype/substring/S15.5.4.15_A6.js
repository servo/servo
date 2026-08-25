// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.substring has not prototype property
es5id: 15.5.4.15_A6
description: Checking String.prototype.substring.prototype
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String.prototype.substring.prototype !== undefined) {
  throw new Test262Error('#1: String.prototype.substring.prototype === undefined. Actual: ' + String.prototype.substring.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////
