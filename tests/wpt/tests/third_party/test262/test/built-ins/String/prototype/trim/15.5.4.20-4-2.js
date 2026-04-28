// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-2
description: >
    String.prototype.trim handles whitepace and lineterminators (
    \u0009abc \u0009)
---*/

assert.sameValue(" \u0009abc \u0009".trim(), "abc", '" \u0009abc \u0009".trim()');
