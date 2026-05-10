// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the valueOf method is 0
es5id: 15.2.4.4_A11
description: Checking the Object.prototype.valueOf.length
---*/
assert(
  !!Object.prototype.valueOf.hasOwnProperty("length"),
  'The value of !!Object.prototype.valueOf.hasOwnProperty("length") is expected to be true'
);

assert.sameValue(
  Object.prototype.valueOf.length,
  0,
  'The value of Object.prototype.valueOf.length is expected to be 0'
);
