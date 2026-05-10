// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The eval property has not prototype property
esid: sec-eval-x
description: Checking eval.prototype
---*/

//CHECK#1
if (eval.prototype !== undefined) {
  throw new Test262Error('#1: eval.prototype === undefined. Actual: ' + (eval.prototype));
}
