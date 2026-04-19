// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-array-prototype-object
info: Array prototype object has a length property
es5id: 15.4.4_A1.3_T1
description: Array.prototype.length === 0
---*/

assert.sameValue(Array.prototype.length, 0, 'The value of Array.prototype.length is expected to be 0');
