// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is NaN, Math.ceil(x) is NaN
es5id: 15.8.2.6_A1
description: Checking if Math.ceil(NaN) is NaN
---*/

assert.sameValue(Math.ceil(NaN), NaN);
