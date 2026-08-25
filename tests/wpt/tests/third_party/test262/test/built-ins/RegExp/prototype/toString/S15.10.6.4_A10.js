// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype.toString.length property has the attribute ReadOnly
es5id: 15.10.6.4_A10
description: >
    Checking if varying the RegExp.prototype.toString.length property
    fails
includes: [propertyHelper.js]
---*/
assert.sameValue(
  RegExp.prototype.toString.hasOwnProperty('length'),
  true,
  'RegExp.prototype.toString.hasOwnProperty(\'length\') must return true'
);

var __obj = RegExp.prototype.toString.length;

verifyNotWritable(RegExp.prototype.toString, "length", null, function(){return "shifted";});

assert.sameValue(
  RegExp.prototype.toString.length,
  __obj,
  'The value of RegExp.prototype.toString.length is expected to equal the value of __obj'
);

// TODO: Convert to verifyProperty() format.
