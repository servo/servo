// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Object prototype object has not prototype
es5id: 15.2.4_A1_T1
description: Checking if obtaining Object.prototype.prototype fails
---*/
assert.sameValue(
  Object.prototype.prototype,
  undefined,
  'The value of Object.prototype.prototype is expected to equal undefined'
);
