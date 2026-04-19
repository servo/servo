// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is NaN, Math.log(x) is NaN
es5id: 15.8.2.10_A1
description: Checking if Math.log(NaN) is NaN
---*/

assert.sameValue(Math.log(NaN), NaN);
