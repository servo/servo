// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-3-19
description: Object.isFrozen returns false for all built-in objects (Error)
---*/

var b = Object.isFrozen(Error);

assert.sameValue(b, false, 'b');
