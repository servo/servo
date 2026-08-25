// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Object.prototype.valueOf.length property does not have the attribute
    DontDelete
es5id: 15.2.4.4_A9
description: >
    Checknig if deleting of the Object.prototype.valueOf.length
    property fails
---*/
assert(
  !!Object.prototype.valueOf.hasOwnProperty('length'),
  'The value of !!Object.prototype.valueOf.hasOwnProperty("length") is expected to be true'
);

assert(
  !!delete Object.prototype.valueOf.length,
  'The value of !!delete Object.prototype.valueOf.length is expected to be true'
);

assert(
  !Object.prototype.valueOf.hasOwnProperty('length'),
  'The value of !Object.prototype.valueOf.hasOwnProperty("length") is expected to be true'
);

// TODO: Convert to verifyProperty() format.
