// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is NaN, Math.round(x) is NaN
es5id: 15.8.2.15_A1
description: Checking if Math.round(x) is NaN, where x is NaN
---*/

assert.sameValue(Math.round(NaN), NaN);
