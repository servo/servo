// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Globally defined variable NaN has not been altered by program execution
es5id: 8.5_A9
description: Try alter globally defined variable NaN
flags: [noStrict]
---*/

Number.NaN = 1;

if (Number.NaN === 1) {
  throw new Test262Error('#1: Globally defined variable NaN has not been altered by program execution');
}
