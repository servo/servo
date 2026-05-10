// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The effect of preventExtensions must be testable by calling isExtensible
es5id: 15.2.3.10-2
description: >
    Object.preventExtensions returns its arguments after setting its
    extensible property to false
---*/

var o = {};
var o2 = undefined;

o2 = Object.preventExtensions(o);

assert.sameValue(o2, o, 'o2');
assert.sameValue(Object.isExtensible(o2), false, 'Object.isExtensible(o2)');
