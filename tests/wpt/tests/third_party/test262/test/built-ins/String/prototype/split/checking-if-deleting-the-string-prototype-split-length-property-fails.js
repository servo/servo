// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The String.prototype.split.length property does not have the attribute
    DontDelete
es5id: 15.5.4.14_A9
description: >
    Checking if deleting the String.prototype.split.length property
    fails
---*/

assert(
  String.prototype.split.hasOwnProperty('length'),
  'String.prototype.split.hasOwnProperty(\'length\') must return true'
);

assert(delete String.prototype.split.length, 'The value of `delete String.prototype.split.length` is true');

assert(
  !String.prototype.split.hasOwnProperty('length'),
  'The value of `!String.prototype.split.hasOwnProperty(\'length\')` is true'
);
