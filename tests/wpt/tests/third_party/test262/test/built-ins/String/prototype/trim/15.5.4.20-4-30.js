// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-30
description: >
    String.prototype.trim handles whitepace and lineterminators
    (\u0020\u0020)
---*/

assert.sameValue("\u0020\u0020".trim(), "", '"\u0020\u0020".trim()');
