// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "getDate"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "getDate"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("getDate"),
  true,
  'Date.prototype.hasOwnProperty("getDate") must return true'
);
