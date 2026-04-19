// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The RegExp.prototype.toString.length property does not have the attribute
    DontDelete
es5id: 15.10.6.4_A9
description: >
    Checking if deleting the RegExp.prototype.toString.length property
    fails
---*/
assert.sameValue(
  RegExp.prototype.toString.hasOwnProperty('length'),
  true,
  'RegExp.prototype.toString.hasOwnProperty(\'length\') must return true'
);

assert.sameValue(
  delete RegExp.prototype.toString.length,
  true,
  'The value of `delete RegExp.prototype.toString.length` is expected to be true'
);

assert.sameValue(
  RegExp.prototype.toString.hasOwnProperty('length'),
  false,
  'RegExp.prototype.toString.hasOwnProperty(\'length\') must return false'
);

// TODO: Convert to verifyProperty() format.
