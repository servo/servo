// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype.exec.length property has the attribute DontEnum
es5id: 15.10.6.2_A8
description: >
    Checking if enumerating the RegExp.prototype.exec.length property
    fails
---*/
assert.sameValue(
  RegExp.prototype.exec.hasOwnProperty('length'),
  true,
  'RegExp.prototype.exec.hasOwnProperty(\'length\') must return true'
);

assert.sameValue(
  RegExp.prototype.exec.propertyIsEnumerable('length'),
  false,
  'RegExp.prototype.exec.propertyIsEnumerable(\'length\') must return false'
);

var count=0;

for (var p in RegExp.prototype.exec){
  if (p==="length") {
    count++;
  }
}

assert.sameValue(count, 0, 'The value of count is expected to be 0');

// TODO: Convert to verifyProperty() format.
