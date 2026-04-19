// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is +Infinity, Math.sin(x) is NaN
es5id: 15.8.2.16_A4
description: Checking if Math.sin(+Infinity) is NaN
---*/

assert.sameValue(Math.sin(Infinity), NaN);
