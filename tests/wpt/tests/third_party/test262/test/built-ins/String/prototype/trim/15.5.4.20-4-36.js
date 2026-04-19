// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-36
description: >
    String.prototype.trim handles whitepace and lineterminators
    (ab\u000Bc)
---*/

assert.sameValue("ab\u000Bc".trim(), "ab\u000Bc", '"ab\u000Bc".trim()');
