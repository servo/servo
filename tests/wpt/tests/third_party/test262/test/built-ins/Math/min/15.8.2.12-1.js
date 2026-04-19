// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.8.2.12-1
description: Math.min({}) is NaN
---*/

assert.sameValue(Math.min({}), NaN);
