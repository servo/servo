// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.MIN_VALUE has the attribute DontEnum
es5id: 15.7.3.3_A4
description: Checking if enumerating Number.MIN_VALUE fails
---*/

for (var x in Number) {
  assert.notSameValue(x, "MIN_VALUE", 'The value of x is not "MIN_VALUE"');
}

assert(
  !Number.propertyIsEnumerable('MIN_VALUE'),
  'The value of !Number.propertyIsEnumerable(\'MIN_VALUE\') is expected to be true'
);

// TODO: Convert to verifyProperty() format.
