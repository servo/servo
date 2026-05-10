// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-3-14
description: >
    String.prototype.trim - 'S' is a string that has null character in
    the middle
---*/

assert.sameValue("a\0\u0000bc".trim(), "a\0\u0000bc", '"a\0\u0000bc".trim()');
