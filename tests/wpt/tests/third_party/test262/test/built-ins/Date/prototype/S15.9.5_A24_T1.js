// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "getMilliseconds"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "getMilliseconds"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("getMilliseconds"),
  true,
  'Date.prototype.hasOwnProperty("getMilliseconds") must return true'
);
