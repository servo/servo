// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Object.prototype.valueOf.length property has the attribute ReadOnly
es5id: 15.2.4.4_A10
description: >
    Checking if varying the Object.prototype.valueOf.length property
    fails
includes: [propertyHelper.js]
---*/
assert(
  !!Object.prototype.valueOf.hasOwnProperty('length'),
  'The value of !!Object.prototype.valueOf.hasOwnProperty("length") is expected to be true'
);

var obj = Object.prototype.valueOf.length;

verifyNotWritable(Object.prototype.valueOf, "length", null, function() {
  return "shifted";
});

assert.sameValue(
  Object.prototype.valueOf.length,
  obj,
  'The value of Object.prototype.valueOf.length is expected to equal the value of obj'
);

// TODO: Convert to verifyProperty() format.
