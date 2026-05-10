// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The String.prototype.split.length property has the attribute DontEnum
es5id: 15.5.4.14_A8
description: >
    Checking if enumerating the String.prototype.split.length property
    fails
---*/

assert(
  String.prototype.split.hasOwnProperty('length'),
  'String.prototype.split.hasOwnProperty(\'length\') must return true'
);

assert(
  !String.prototype.split.propertyIsEnumerable('length'),
  'The value of `!String.prototype.split.propertyIsEnumerable(\'length\')` is true'
);

//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
// CHECK#2
var count = 0;

for (var p in String.prototype.split) {
  if (p === "length") {
    count++;
  }
}

assert.sameValue(count, 0, 'The value of `count` is 0');
