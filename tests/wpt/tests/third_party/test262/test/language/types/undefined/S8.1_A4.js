// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If property of object not exist, return undefined
es5id: 8.1_A4
description: Check value of not existed property
---*/

// CHECK#1
if ((new Object()).newProperty !== undefined) {
  throw new Test262Error('#1: (new Object()).newProperty === undefined. Actual: ' + ((new Object()).newProperty));
}
