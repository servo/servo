// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The join property of Array has not prototype property
esid: sec-array.prototype.join
description: Checking Array.prototype.join.prototype
---*/

if (Array.prototype.join.prototype !== undefined) {
  throw new Test262Error('#1: Array.prototype.join.prototype === undefined. Actual: ' + (Array.prototype.join.prototype));
}
