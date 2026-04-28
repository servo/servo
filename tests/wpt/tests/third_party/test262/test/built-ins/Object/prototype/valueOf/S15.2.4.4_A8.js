// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Object.prototype.valueOf.length property has the attribute DontEnum
es5id: 15.2.4.4_A8
description: >
    Checking if enumerating the Object.prototype.valueOf.length
    property fails
---*/
assert(
  !!Object.prototype.valueOf.hasOwnProperty('length'),
  'The value of !!Object.prototype.valueOf.hasOwnProperty("length") is expected to be true'
);

assert(
  !Object.prototype.valueOf.propertyIsEnumerable('length'),
  'The value of !Object.prototype.valueOf.propertyIsEnumerable("length") is expected to be true'
);

for (var p in Object.prototype.valueOf) {
  assert.notSameValue(p, "length", 'The value of p is not "length"');
}
//

// TODO: Convert to verifyProperty() format.
