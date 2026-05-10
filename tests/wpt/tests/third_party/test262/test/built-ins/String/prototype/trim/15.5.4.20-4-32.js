// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-4-32
description: >
    String.prototype.trim handles whitepace and lineterminators
    (\u00A0\u00A0)
---*/

assert.sameValue("\u00A0\u00A0".trim(), "", '"\u00A0\u00A0".trim()');
