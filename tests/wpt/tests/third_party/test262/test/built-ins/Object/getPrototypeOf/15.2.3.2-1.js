// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-1
description: Object.getPrototypeOf returns Number.prototype if 'O' is a number
---*/

assert.sameValue(Object.getPrototypeOf(0), Number.prototype, 'Object.getPrototypeOf(0)');
