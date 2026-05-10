// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Since Error prototype object is not function it has no [[Construct]] method
es5id: 15.11.4_A4
description: Checking if creating "new Error.prototype" fails
---*/

assert.throws(TypeError, () => {
  new Error.prototype();
  throw new Test262Error();
});
