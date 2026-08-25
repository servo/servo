// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Created based on feedback in
    https://bugs.ecmascript.org/show_bug.cgi?id=333
es5id: 10.4.3-1-105
description: >
    Non strict mode should ToObject thisArg if not an object.  Return
    type should be object and strict equality should fail.
flags: [noStrict]
---*/

Object.defineProperty(Object.prototype, "x", { get: function () { return this; } });

assert.sameValue((5).x === 5, false, '(5).x === 5');
assert.sameValue(typeof (5).x, "object", 'typeof (5).x');
