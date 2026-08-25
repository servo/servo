// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: A property can have attribute ReadOnly like E in Math
es5id: 8.6.1_A1
description: Try change Math.E property
flags: [noStrict]
---*/

var __e = Math.E;
Math.E=1;
if (Math.E !==__e){
  throw new Test262Error('#1: __e = Math.E; Math.E=1; Math.E ===__e');
}
