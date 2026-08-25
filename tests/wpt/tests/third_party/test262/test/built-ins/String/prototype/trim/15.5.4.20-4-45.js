// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-45
description: >
    String.prototype.trim handles whitepace and lineterminators
    (\u2028abc)
---*/

assert.sameValue("\u2028abc".trim(), "abc", '"\u2028abc".trim()');
