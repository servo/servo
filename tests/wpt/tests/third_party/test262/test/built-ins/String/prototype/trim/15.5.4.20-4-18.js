// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-18
description: >
    String.prototype.trim handles whitepace and lineterminators
    (abc\uFEFF)
---*/

assert.sameValue("abc\uFEFF".trim(), "abc", '"abc\uFEFF".trim()');
