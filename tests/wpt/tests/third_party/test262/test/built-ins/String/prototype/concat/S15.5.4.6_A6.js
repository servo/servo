// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.concat has not prototype property
es5id: 15.5.4.6_A6
description: Checking String.prototype.concat.prototype
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String.prototype.concat.prototype !== undefined) {
  throw new Test262Error('#1: String.prototype.concat.prototype === undefined. Actual: ' + String.prototype.concat.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////
