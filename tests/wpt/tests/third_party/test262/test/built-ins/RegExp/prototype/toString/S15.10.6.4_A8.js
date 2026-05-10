// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype.toString.length property has the attribute DontEnum
es5id: 15.10.6.4_A8
description: >
    Checking if enumerating the RegExp.prototype.toString.length
    property fails
---*/
assert.sameValue(
  RegExp.prototype.toString.hasOwnProperty('length'),
  true,
  'RegExp.prototype.toString.hasOwnProperty(\'length\') must return true'
);

assert.sameValue(
  RegExp.prototype.toString.propertyIsEnumerable('length'),
  false,
  'RegExp.prototype.toString.propertyIsEnumerable(\'length\') must return false'
);

var count=0;

for (var p in RegExp.prototype.toString){
    if (p==="length") {
      count++;
    }
}

assert.sameValue(count, 0, 'The value of count is expected to be 0');

// TODO: Convert to verifyProperty() format.
