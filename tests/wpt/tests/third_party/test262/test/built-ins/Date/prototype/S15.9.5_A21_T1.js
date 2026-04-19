// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "getUTCMinutes"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "getUTCMinutes"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("getUTCMinutes"),
  true,
  'Date.prototype.hasOwnProperty("getUTCMinutes") must return true'
);
