// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.11-4-13
description: >
    Object.isSealed returns false for all built-in objects
    (Number.prototype)
---*/

var b = Object.isSealed(Number.prototype);

assert.sameValue(b, false, 'b');
