// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is NaN, Math.abs(x) is NaN
es5id: 15.8.2.1_A1
description: Checking if Math.abs(NaN) is NaN
---*/

assert.sameValue(Math.abs(NaN), NaN);
