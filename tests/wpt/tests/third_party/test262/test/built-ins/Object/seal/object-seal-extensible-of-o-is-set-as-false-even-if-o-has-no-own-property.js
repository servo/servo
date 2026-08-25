// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.seal
description: >
    Object.seal - extensible of 'O' is set as false even if 'O' has no
    own property
---*/

var obj = {};

var preCheck = Object.isExtensible(obj);

Object.seal(obj);

assert(preCheck, 'preCheck !== true');
assert.sameValue(Object.isExtensible(obj), false, 'Object.isExtensible(obj)');
