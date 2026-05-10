// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-39
description: >
    String.prototype.trim handles whitepace and lineterminators
    (ab\u0085c)
---*/

assert.sameValue("ab\u0085c".trim(), "ab\u0085c", '"ab\u0085c".trim()');
