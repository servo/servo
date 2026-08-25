// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Function.prototype.toString.length property has the attribute ReadOnly
es5id: 15.3.4.2_A10
description: >
    Checking if varying the Function.prototype.toString.length
    property fails
includes: [propertyHelper.js]
---*/
assert(
  Function.prototype.toString.hasOwnProperty('length'),
  'Function.prototype.toString.hasOwnProperty(\'length\') must return true'
);

var obj = Function.prototype.toString.length;

verifyNotWritable(Function.prototype.toString, "length", null, function(){return "shifted";});

assert.sameValue(
  Function.prototype.toString.length,
  obj,
  'The value of Function.prototype.toString.length is expected to equal the value of obj'
);

// TODO: Convert to verifyProperty() format.
