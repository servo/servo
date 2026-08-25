// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - 'O' is frozen already
---*/

var obj = {};

obj.foo = 10; // default value of attributes: writable: true, configurable: true, enumerable: true
var preCheck = Object.isExtensible(obj);
Object.freeze(obj);

Object.seal(obj);

assert(preCheck, 'preCheck !== true');
assert(Object.isSealed(obj), 'Object.isSealed(obj) !== true');
