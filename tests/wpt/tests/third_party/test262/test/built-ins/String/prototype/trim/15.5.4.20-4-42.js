// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-42
description: >
    String.prototype.trim handles whitepace and lineterminators
    (ab\uFEFFc)
---*/

assert.sameValue("ab\uFEFFc".trim(), "ab\uFEFFc", '"ab\uFEFFc".trim()');
