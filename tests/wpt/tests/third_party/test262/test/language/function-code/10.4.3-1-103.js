// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-103
description: >
    Non strict mode should ToObject thisArg if not an object.
    Abstract equality operator should succeed.
---*/

Object.defineProperty(Object.prototype, "x", { get: function () { return this; } });

assert.sameValue((5).x == 0, false, '(5).x == 0');
assert((5).x == 5, '(5).x == 5');
