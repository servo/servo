// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "getUTCFullYear"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "getUTCFullYear"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("getUTCFullYear"),
  true,
  'Date.prototype.hasOwnProperty("getUTCFullYear") must return true'
);
