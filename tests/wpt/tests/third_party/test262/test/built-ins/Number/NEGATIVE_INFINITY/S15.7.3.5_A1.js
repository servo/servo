// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.NEGATIVE_INFINITY is -Infinity
es5id: 15.7.3.5_A1
description: Checking sign and finiteness of Number.NEGATIVE_INFINITY
---*/
assert.sameValue(isFinite(Number.NEGATIVE_INFINITY), false, 'isFinite(Number.NEGATIVE_INFINITY) must return false');
