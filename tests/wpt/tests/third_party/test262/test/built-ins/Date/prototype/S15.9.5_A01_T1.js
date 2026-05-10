// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "constructor"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "constructor"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("constructor"),
  true,
  'Date.prototype.hasOwnProperty("constructor") must return true'
);
