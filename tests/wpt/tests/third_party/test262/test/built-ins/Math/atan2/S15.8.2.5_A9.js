// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If y is -0 and x is +0, Math.atan2(y,x) is -0
es5id: 15.8.2.5_A9
description: Checking if Math.atan2(y,x) is -0, where y is -0 and x is +0
---*/

assert.sameValue(Math.atan2(-0, 0), -0);
