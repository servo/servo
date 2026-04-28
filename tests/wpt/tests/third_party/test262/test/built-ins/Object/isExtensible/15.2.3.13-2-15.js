// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.13-2-15
description: >
    Object.isExtensible returns true for all built-in objects
    (Array.prototype)
---*/

var e = Object.isExtensible(Array.prototype);

assert.sameValue(e, true, 'e');
