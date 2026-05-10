// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is -0, Math.abs(x) is +0
es5id: 15.8.2.1_A2
description: Checking if Math.abs(-0) equals to +0
---*/

assert.sameValue(Math.abs(-0), 0);
