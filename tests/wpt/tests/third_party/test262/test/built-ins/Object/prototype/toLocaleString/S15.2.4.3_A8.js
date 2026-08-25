// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Object.prototype.toLocaleString.length property has the attribute
    DontEnum
es5id: 15.2.4.3_A8
description: >
    Checking if enumerating the Object.prototype.toLocaleString.length
    property fails
---*/
assert(
  !!Object.prototype.toLocaleString.hasOwnProperty('length'),
  'The value of !!Object.prototype.toLocaleString.hasOwnProperty("length") is expected to be true'
);

assert(
  !Object.prototype.toLocaleString.propertyIsEnumerable('length'),
  'The value of !Object.prototype.toLocaleString.propertyIsEnumerable("length") is expected to be true'
);

for (var p in Object.prototype.toLocaleString) {
  assert.notSameValue(p, "length", 'The value of p is not "length"');
}
//

// TODO: Convert to verifyProperty() format.
