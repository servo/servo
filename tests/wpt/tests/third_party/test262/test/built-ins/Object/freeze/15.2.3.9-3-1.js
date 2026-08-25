// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-3-1
description: Object.freeze - returned object is not extensible
---*/

var obj = {};
Object.freeze(obj);

assert.sameValue(Object.isExtensible(obj), false, 'Object.isExtensible(obj)');
