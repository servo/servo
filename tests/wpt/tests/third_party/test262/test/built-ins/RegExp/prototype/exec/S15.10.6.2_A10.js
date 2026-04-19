// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype.exec.length property has the attribute ReadOnly
es5id: 15.10.6.2_A10
description: Checking if varying the RegExp.prototype.exec.length property fails
includes: [propertyHelper.js]
---*/
assert.sameValue(
  RegExp.prototype.exec.hasOwnProperty('length'),
  true,
  'RegExp.prototype.exec.hasOwnProperty(\'length\') must return true'
);

var __obj = RegExp.prototype.exec.length;

verifyNotWritable(RegExp.prototype.exec, "length", null, function(){return "shifted";});

assert.sameValue(
  RegExp.prototype.exec.length,
  __obj,
  'The value of RegExp.prototype.exec.length is expected to equal the value of __obj'
);

// TODO: Convert to verifyProperty() format.
