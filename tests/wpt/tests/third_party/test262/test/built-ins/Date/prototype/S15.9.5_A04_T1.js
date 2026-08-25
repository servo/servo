// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "toTimeString"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "toTimeString"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("toTimeString"),
  true,
  'Date.prototype.hasOwnProperty("toTimeString") must return true'
);
