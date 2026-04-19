// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Function.prototype.toString.length property does not have the
    attribute DontDelete
es5id: 15.3.4.2_A9
description: >
    Checking if deleting the Function.prototype.toString.length
    property fails
---*/
assert(
  Function.prototype.toString.hasOwnProperty('length'),
  'Function.prototype.toString.hasOwnProperty(\'length\') must return true'
);

assert(
  delete Function.prototype.toString.length,
  'The value of delete Function.prototype.toString.length is expected to be true'
);

assert(
  !Function.prototype.toString.hasOwnProperty('length'),
  'The value of !Function.prototype.toString.hasOwnProperty(\'length\') is expected to be true'
);

// TODO: Convert to verifyProperty() format.
