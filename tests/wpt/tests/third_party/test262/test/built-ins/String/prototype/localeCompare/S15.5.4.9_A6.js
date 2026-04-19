// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.localeCompare has not prototype property
es5id: 15.5.4.9_A6
description: Checking String.prototype.localeCompare.prototype
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String.prototype.localeCompare.prototype !== undefined) {
  throw new Test262Error('#1: String.prototype.localeCompare.prototype === undefined. Actual: ' + String.prototype.localeCompare.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////
