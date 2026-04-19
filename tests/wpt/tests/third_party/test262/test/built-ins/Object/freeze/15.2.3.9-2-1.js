// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-1
description: >
    Object.freeze - extensible of 'O' is set as false even if 'O' has
    no own property
---*/

var obj = {};

Object.freeze(obj);

assert.sameValue(Object.isExtensible(obj), false, 'Object.isExtensible(obj)');
