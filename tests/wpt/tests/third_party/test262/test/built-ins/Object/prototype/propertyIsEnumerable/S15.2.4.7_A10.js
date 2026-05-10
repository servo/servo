// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Object.prototype.propertyIsEnumerable.length property has the
    attribute ReadOnly
es5id: 15.2.4.7_A10
description: >
    Checking if varying the
    Object.prototype.propertyIsEnumerable.length property fails
includes: [propertyHelper.js]
---*/
assert(
  !!Object.prototype.propertyIsEnumerable.hasOwnProperty('length'),
  'The value of !!Object.prototype.propertyIsEnumerable.hasOwnProperty("length") is expected to be true'
);

var obj = Object.prototype.propertyIsEnumerable.length;

verifyNotWritable(Object.prototype.propertyIsEnumerable, "length", null, function() {
  return "shifted";
});

assert.sameValue(
  Object.prototype.propertyIsEnumerable.length,
  obj,
  'The value of Object.prototype.propertyIsEnumerable.length is expected to equal the value of obj'
);

// TODO: Convert to verifyProperty() format.
