// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Function.prototype.toString.length property has the attribute DontEnum
es5id: 15.3.4.2_A8
description: >
    Checking if enumerating the Function.prototype.toString.length
    property fails
---*/
assert(
  Function.prototype.toString.hasOwnProperty('length'),
  'Function.prototype.toString.hasOwnProperty(\'length\') must return true'
);

assert(
  !Function.prototype.toString.propertyIsEnumerable('length'),
  'The value of !Function.prototype.toString.propertyIsEnumerable(\'length\') is expected to be true'
);

// CHECK#2
for (var p in Function.prototype.toString){
  assert.notSameValue(p, "length", 'The value of p is not "length"');
}

// TODO: Convert to verifyProperty() format.
