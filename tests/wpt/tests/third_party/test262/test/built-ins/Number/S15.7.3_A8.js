// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number constructor has length property whose value is 1
es5id: 15.7.3_A8
description: Checking Number.length property
---*/
assert(Number.hasOwnProperty("length"), 'Number.hasOwnProperty("length") must return true');
assert.sameValue(Number.length, 1, 'The value of Number.length is expected to be 1');
