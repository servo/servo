// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Created based on feedback in
    https://bugs.ecmascript.org/show_bug.cgi?id=333
es5id: 10.4.3-1-106
description: >
    Strict mode should not ToObject thisArg if not an object.  Return
    type should be 'number'.
flags: [onlyStrict]
---*/

Object.defineProperty(Object.prototype, "x", { get: function () { return this; } });

assert.sameValue(typeof (5).x, "number", 'typeof (5).x');
