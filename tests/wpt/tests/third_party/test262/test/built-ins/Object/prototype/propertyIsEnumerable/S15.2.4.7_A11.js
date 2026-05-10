// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the hasOwnProperty method is 1
es5id: 15.2.4.7_A11
description: Checking the value of Object.prototype.hasOwnProperty.length
---*/
assert(
  !!Object.prototype.propertyIsEnumerable.hasOwnProperty("length"),
  'The value of !!Object.prototype.propertyIsEnumerable.hasOwnProperty("length") is expected to be true'
);

assert.sameValue(
  Object.prototype.propertyIsEnumerable.length,
  1,
  'The value of Object.prototype.propertyIsEnumerable.length is expected to be 1'
);
