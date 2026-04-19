// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.split.length property has the attribute ReadOnly
es5id: 15.5.4.14_A10
description: >
    Checking if varying the String.prototype.split.length property
    fails
includes: [propertyHelper.js]
---*/

assert(
  String.prototype.split.hasOwnProperty('length'),
  'String.prototype.split.hasOwnProperty(\'length\') must return true'
);

var __obj = String.prototype.split.length;

verifyNotWritable(String.prototype.split, "length", null, function() {
  return "shifted";
});

assert.sameValue(
  String.prototype.split.length,
  __obj,
  'The value of String.prototype.split.length is expected to equal the value of __obj'
);
