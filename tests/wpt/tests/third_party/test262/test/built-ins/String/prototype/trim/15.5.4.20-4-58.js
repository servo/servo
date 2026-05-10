// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-58
description: >
    String.prototype.trim handles whitepace and lineterminators
    (\u2029\u2029)
---*/

assert.sameValue("\u2029\u2029".trim(), "", '"\u2029\u2029".trim()');
