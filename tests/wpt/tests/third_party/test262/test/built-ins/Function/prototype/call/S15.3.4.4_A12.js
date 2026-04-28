// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function.prototype.call has not prototype property
es5id: 15.3.4.4_A12
description: >
    Checking if obtaining the prototype property of
    Function.prototype.call fails
---*/
assert.sameValue(
  Function.prototype.call.prototype,
  undefined,
  'The value of Function.prototype.call.prototype is expected to equal undefined'
);
