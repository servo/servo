// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-4-3
description: Object.freeze - the extensions of 'O' is prevented already
---*/

var obj = {};

obj.foo = 10; // default value of attributes: writable: true, enumerable: true

Object.preventExtensions(obj);

Object.freeze(obj);

assert(Object.isFrozen(obj), 'Object.isFrozen(obj) !== true');
