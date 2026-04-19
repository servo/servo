// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Number prototype object has the property toFixed
es5id: 15.7.4_A3.5
description: The test uses hasOwnProperty() method
---*/
assert.sameValue(
  Number.prototype.hasOwnProperty("toFixed"),
  true,
  'Number.prototype.hasOwnProperty("toFixed") must return true'
);
