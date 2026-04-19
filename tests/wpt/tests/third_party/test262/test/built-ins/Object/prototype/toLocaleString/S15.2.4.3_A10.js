// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Object.prototype.toLocaleString.length property has the attribute
    ReadOnly
es5id: 15.2.4.3_A10
description: >
    Checking if varying the Object.prototype.toLocaleString.length
    property fails
includes: [propertyHelper.js]
---*/
assert(
  !!Object.prototype.toLocaleString.hasOwnProperty('length'),
  'The value of !!Object.prototype.toLocaleString.hasOwnProperty("length") is expected to be true'
);

var obj = Object.prototype.toLocaleString.length;

verifyNotWritable(Object.prototype.toLocaleString, "length", null, function() {
  return "shifted";
});

assert.sameValue(
  Object.prototype.toLocaleString.length,
  obj,
  'The value of Object.prototype.toLocaleString.length is expected to equal the value of obj'
);

// TODO: Convert to verifyProperty() format.
