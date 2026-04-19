// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charCodeAt has not prototype property
es5id: 15.5.4.5_A6
description: Checking String.prototype.charCodeAt.prototype
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String.prototype.charCodeAt.prototype !== undefined) {
  throw new Test262Error('#1: String.prototype.charCodeAt.prototype === undefined. Actual: ' + String.prototype.charCodeAt.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////
