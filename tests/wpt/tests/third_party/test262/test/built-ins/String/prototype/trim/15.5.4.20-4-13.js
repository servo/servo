// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-13
description: >
    String.prototype.trim handles whitepace and lineterminators
    (abc\u000C)
---*/

assert.sameValue("abc\u000C".trim(), "abc", '"abc\u000C".trim()');
