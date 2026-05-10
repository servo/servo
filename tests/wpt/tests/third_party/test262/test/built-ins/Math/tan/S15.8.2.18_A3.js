// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is -0, Math.tan(x) is -0
es5id: 15.8.2.18_A3
description: Checking if Math.tan(-0) equals to -0
---*/

assert.sameValue(Math.tan(-0), -0);
