// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.12-3-9
description: >
    Object.isFrozen returns false for all built-in objects
    (String.prototype)
---*/

var b = Object.isFrozen(String.prototype);

assert.sameValue(b, false, 'b');
