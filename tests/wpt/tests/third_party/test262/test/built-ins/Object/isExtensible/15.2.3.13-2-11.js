// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.13-2-11
description: Object.isExtensible returns true for all built-in objects (Error)
---*/

var e = Object.isExtensible(Error);

assert.sameValue(e, true, 'e');
