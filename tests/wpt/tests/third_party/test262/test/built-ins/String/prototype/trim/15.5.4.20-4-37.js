// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-37
description: >
    String.prototype.trim handles whitepace and lineterminators
    (ab\u000Cc)
---*/

assert.sameValue("ab\u000Cc".trim(), "ab\u000Cc", '"ab\u000Cc".trim()');
