// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Object.prototype.propertyIsEnumerable.length property has the
    attribute DontEnum
es5id: 15.2.4.7_A8
description: >
    Checking if enumerating the
    Object.prototype.propertyIsEnumerable.length property fails
---*/
assert(
  !!Object.prototype.propertyIsEnumerable.hasOwnProperty('length'),
  'The value of !!Object.prototype.propertyIsEnumerable.hasOwnProperty("length") is expected to be true'
);

assert(
  !Object.prototype.propertyIsEnumerable.propertyIsEnumerable('length'),
  'The value of !Object.prototype.propertyIsEnumerable.propertyIsEnumerable("length") is expected to be true'
);

for (var p in Object.prototype.propertyIsEnumerable) {
  assert.notSameValue(p, "length", 'The value of p is not "length"');
}
//

// TODO: Convert to verifyProperty() format.
