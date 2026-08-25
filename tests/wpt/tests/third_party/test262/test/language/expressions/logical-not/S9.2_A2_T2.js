// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of boolean conversion from null value is false
es5id: 9.2_A2_T2
description: null convert to Boolean by implicit transformation
---*/

// CHECK#1
if (!(null) !== true) {
  throw new Test262Error('#1: !(null) === true. Actual: ' + (!(null))); 
}
