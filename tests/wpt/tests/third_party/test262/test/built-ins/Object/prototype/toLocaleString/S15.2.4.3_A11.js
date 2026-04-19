// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the toLocaleString method is 0
es5id: 15.2.4.3_A11
description: Checking the Object.prototype.toLocaleString.length
---*/
assert(
  !!Object.prototype.toLocaleString.hasOwnProperty("length"),
  'The value of !!Object.prototype.toLocaleString.hasOwnProperty("length") is expected to be true'
);

assert.sameValue(
  Object.prototype.toLocaleString.length,
  0,
  'The value of Object.prototype.toLocaleString.length is expected to be 0'
);
