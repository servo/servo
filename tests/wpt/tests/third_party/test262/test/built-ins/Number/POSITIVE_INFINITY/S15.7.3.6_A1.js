// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.POSITIVE_INFINITY is +Infinity
es5id: 15.7.3.6_A1
description: Checking sign and finiteness of Number.POSITIVE_INFINITY
---*/
assert.sameValue(isFinite(Number.POSITIVE_INFINITY), false, 'isFinite(Number.POSITIVE_INFINITY) must return false');
