// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "getUTCMonth"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "getUTCMonth"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("getUTCMonth"),
  true,
  'Date.prototype.hasOwnProperty("getUTCMonth") must return true'
);
