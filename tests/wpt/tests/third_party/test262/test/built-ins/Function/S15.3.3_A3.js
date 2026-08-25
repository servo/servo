// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function constructor has length property whose value is 1
es5id: 15.3.3_A3
description: Checking Function.length property
---*/
assert(Function.hasOwnProperty("length"), 'Function.hasOwnProperty("length") must return true');
assert.sameValue(Function.length, 1, 'The value of Function.length is expected to be 1');
