// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Since Error prototype object is not function it has no [[Call]] method
es5id: 15.11.4_A3
description: Checking if call of Error prototype as a function fails
---*/

assert.throws(TypeError, () => {
  Error.prototype();
  throw new Test262Error();
});
