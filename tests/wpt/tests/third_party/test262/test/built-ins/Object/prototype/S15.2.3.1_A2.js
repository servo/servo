// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Object.prototype property has the attribute DontEnum
es5id: 15.2.3.1_A2
description: Checking if enumerating "Object.prototype" property fails
---*/
assert(
  !Object.propertyIsEnumerable('prototype'),
  'The value of !Object.propertyIsEnumerable("prototype") is expected to be true'
);

var cout = 0;

for (var p in Object) {
  if (p === "prototype") {
    cout++;
  }
}

assert.sameValue(cout, 0, 'The value of cout is expected to be 0');

// TODO: Convert to verifyProperty() format.
