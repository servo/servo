// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Number prototype object has the property valueOf
es5id: 15.7.4_A3.4
description: The test uses hasOwnProperty() method
---*/
assert.sameValue(
  Number.prototype.hasOwnProperty("valueOf"),
  true,
  'Number.prototype.hasOwnProperty("valueOf") must return true'
);
