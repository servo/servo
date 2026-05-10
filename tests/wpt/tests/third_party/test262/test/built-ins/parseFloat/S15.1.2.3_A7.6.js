// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The parseFloat property has not prototype property
esid: sec-parsefloat-string
description: Checking parseFloat.prototype
---*/

//CHECK#1
if (parseFloat.prototype !== undefined) {
  throw new Test262Error('#1: parseFloat.prototype === undefined. Actual: ' + (parseFloat.prototype));
}
