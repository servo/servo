// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.2-1-4
description: Object.getPrototypeOf returns String.prototype if 'O' is a string
---*/

assert.sameValue(Object.getPrototypeOf("abc"), String.prototype, 'Object.getPrototypeOf("abc")');
