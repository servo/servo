// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Object constructor has length property whose value is 1
es5id: 15.2.3_A3
description: Checking Object.length
---*/
assert(
  Object.prototype.hasOwnProperty.call(Object, "length"),
  "The Object constructor has a 'length' own property"
);

assert.sameValue(Object.length, 1, 'The value of Object.length is expected to be 1');
