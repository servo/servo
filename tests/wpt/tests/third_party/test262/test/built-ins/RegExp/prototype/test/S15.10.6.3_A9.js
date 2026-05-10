// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The RegExp.prototype.test.length property does not have the attribute
    DontDelete
es5id: 15.10.6.3_A9
description: Checking if deleting RegExp.prototype.test.length property fails
---*/
assert.sameValue(
  RegExp.prototype.exec.hasOwnProperty('length'),
  true,
  'RegExp.prototype.exec.hasOwnProperty(\'length\') must return true'
);

assert.sameValue(
  delete RegExp.prototype.exec.length,
  true,
  'The value of `delete RegExp.prototype.exec.length` is expected to be true'
);

assert.sameValue(
  RegExp.prototype.exec.hasOwnProperty('length'),
  false,
  'RegExp.prototype.exec.hasOwnProperty(\'length\') must return false'
);

// TODO: Convert to verifyProperty() format.
