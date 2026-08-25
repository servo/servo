// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "setUTCFullYear"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "setUTCFullYear"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("setUTCFullYear"),
  true,
  'Date.prototype.hasOwnProperty("setUTCFullYear") must return true'
);
