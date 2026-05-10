// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-3-7
description: >
    Object.isFrozen returns false for all built-in objects
    (Array.prototype)
---*/

var b = Object.isFrozen(Array.prototype);

assert.sameValue(b, false, 'b');
