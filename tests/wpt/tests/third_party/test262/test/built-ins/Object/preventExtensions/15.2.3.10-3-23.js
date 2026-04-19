// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-3-23
description: >
    Object.preventExtensions - properties can still be reassigned
    after extensions have been prevented
---*/

var obj = {
  prop: 12
};
var preCheck = Object.isExtensible(obj);
Object.preventExtensions(obj);

obj.prop = -1;

assert(preCheck, 'preCheck !== true');
assert.sameValue(obj.prop, -1, 'obj.prop');
