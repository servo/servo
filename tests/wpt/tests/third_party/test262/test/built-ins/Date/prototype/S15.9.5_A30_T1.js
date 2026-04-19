// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Date.prototype has the property "setSeconds"
esid: sec-properties-of-the-date-prototype-object
description: The Date.prototype has the property "setSeconds"
---*/
assert.sameValue(
  Date.prototype.hasOwnProperty("setSeconds"),
  true,
  'Date.prototype.hasOwnProperty("setSeconds") must return true'
);
