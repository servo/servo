// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Date constructor has length property whose value is 7
esid: sec-date-constructor
description: Checking Date.length property
---*/
assert(Date.hasOwnProperty("length"), 'Date.hasOwnProperty("length") must return true');
assert.sameValue(Date.length, 7, 'The value of Date.length is expected to be 7');
