// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the toFixed method is 1
es5id: 15.7.4.5_A2_T01
description: Checking Number prototype itself
---*/
assert.sameValue(
  Number.prototype.toFixed.hasOwnProperty("length"),
  true,
  'Number.prototype.toFixed.hasOwnProperty("length") must return true'
);

assert.sameValue(
  Number.prototype.toFixed.length,
  1,
  'The value of Number.prototype.toFixed.length is expected to be 1'
);
