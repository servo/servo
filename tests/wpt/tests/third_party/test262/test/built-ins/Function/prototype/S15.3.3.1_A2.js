// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Function.prototype property has the attribute DontEnum
es5id: 15.3.3.1_A2
description: Checking if enumerating the Function.prototype property fails
---*/
assert(
  !Function.propertyIsEnumerable('prototype'),
  'The value of !Function.propertyIsEnumerable(\'prototype\') is expected to be true'
);

// CHECK#2
var count = 0;

for (var p in Function) {
  if (p === "prototype") {
    count++;
  }
}

assert.sameValue(count, 0, 'The value of count is expected to be 0');

// TODO: Convert to verifyProperty() format.
