// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function.prototype.toString has not prototype property
es5id: 15.3.4.2_A6
description: >
    Checking if obtaining the prototype property of
    Function.prototype.toString fails
---*/
assert.sameValue(
  Function.prototype.toString.prototype,
  undefined,
  'The value of Function.prototype.toString.prototype is expected to equal undefined'
);
