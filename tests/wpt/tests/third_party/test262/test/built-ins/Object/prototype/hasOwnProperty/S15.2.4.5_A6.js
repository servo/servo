// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Object.prototype.hasOwnProperty has not prototype property
es5id: 15.2.4.5_A6
description: >
    Checking if obtaining the prototype property of
    Object.prototype.hasOwnProperty fails
---*/
assert.sameValue(
  Object.prototype.hasOwnProperty.prototype,
  undefined,
  'The value of Object.prototype.hasOwnProperty.prototype is expected to equal undefined'
);
