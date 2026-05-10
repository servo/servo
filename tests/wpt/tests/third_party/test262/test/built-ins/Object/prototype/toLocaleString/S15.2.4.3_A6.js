// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Object.prototype.toLocaleString has not prototype property
es5id: 15.2.4.3_A6
description: >
    Checking if obtaining the prototype property of
    Object.prototype.toLocaleString fails
---*/
assert.sameValue(
  Object.prototype.toLocaleString.prototype,
  undefined,
  'The value of Object.prototype.toLocaleString.prototype is expected to equal undefined'
);
