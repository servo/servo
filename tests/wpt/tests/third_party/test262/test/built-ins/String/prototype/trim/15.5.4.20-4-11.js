// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-11
description: >
    String.prototype.trim handles whitepace and lineterminators
    (abc\u0009)
---*/

assert.sameValue("abc\u0009".trim(), "abc", '"abc\u0009".trim()');
