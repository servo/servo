// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype.test.length property has the attribute ReadOnly
es5id: 15.10.6.3_A10
description: Checking if varying the RegExp.prototype.test.length property fails
includes: [propertyHelper.js]
---*/
assert.sameValue(
  RegExp.prototype.test.hasOwnProperty('length'),
  true,
  'RegExp.prototype.test.hasOwnProperty(\'length\') must return true'
);

var __obj = RegExp.prototype.test.length;

verifyNotWritable(RegExp.prototype.test, "length", null, function(){return "shifted";});

assert.sameValue(
  RegExp.prototype.test.length,
  __obj,
  'The value of RegExp.prototype.test.length is expected to equal the value of __obj'
);

// TODO: Convert to verifyProperty() format.
