// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-3-8
description: String.prototype.trim - 'S' is a string with all null character
---*/

assert.sameValue("\0\u0000".trim(), "\0\u0000", '"\0\u0000".trim()');
