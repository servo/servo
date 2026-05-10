// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-104
description: >
    Strict mode should not ToObject thisArg if not an object.  Strict
    equality operator should succeed.
flags: [onlyStrict]
---*/

Object.defineProperty(Object.prototype, "x", { get: function () { return this; } });

assert((5).x === 5, '(5).x === 5');
