// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "toUTCString"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "toUTCString"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("toUTCString"),
  true,
  'Date.prototype.hasOwnProperty("toUTCString") must return true'
);
