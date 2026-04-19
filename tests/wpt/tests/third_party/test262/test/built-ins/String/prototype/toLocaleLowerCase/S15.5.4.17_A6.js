// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleLowerCase has not prototype property
es5id: 15.5.4.17_A6
description: Checking String.prototype.toLocaleLowerCase.prototype
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String.prototype.toLocaleLowerCase.prototype !== undefined) {
  throw new Test262Error('#1: String.prototype.toLocaleLowerCase.prototype === undefined. Actual: ' + String.prototype.toLocaleLowerCase.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////
