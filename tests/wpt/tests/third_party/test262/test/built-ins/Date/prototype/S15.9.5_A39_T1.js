// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "setUTCMonth"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "setUTCMonth"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("setUTCMonth"),
  true,
  'Date.prototype.hasOwnProperty("setUTCMonth") must return true'
);
