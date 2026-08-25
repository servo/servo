// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.13-2-14
description: >
    Object.isExtensible returns true for all built-in objects
    (Function.prototype)
---*/

var e = Object.isExtensible(Function.prototype);

assert.sameValue(e, true, 'e');
