// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-57
description: >
    String.prototype.trim handles whitepace and lineterminators
    (\u2028\u2028)
---*/

assert.sameValue("\u2028\u2028".trim(), "", '"\u2028\u2028".trim()');
