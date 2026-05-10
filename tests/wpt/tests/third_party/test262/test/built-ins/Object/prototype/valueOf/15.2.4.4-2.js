// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.4.4-2
description: >
    Object.prototype.valueOf - typeof
    Object.prototype.valueOf.call(false)==="object"
---*/

assert.sameValue(typeof Object.prototype.valueOf.call(false), "object", 'typeof Object.prototype.valueOf.call(false)');
