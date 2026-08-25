// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-3-9
description: String.prototype.trim - 'S' is a string with null character ('\0')
---*/

assert.sameValue("\0".trim(), "\0", '"\0".trim()');
