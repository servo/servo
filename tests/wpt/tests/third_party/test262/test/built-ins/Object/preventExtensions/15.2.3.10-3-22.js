// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-3-22
description: >
    Object.preventExtensions - properties can still be deleted after
    extensions have been prevented
---*/

var obj = {
  prop: 12
};
var preCheck = Object.isExtensible(obj);
Object.preventExtensions(obj);

delete obj.prop;

assert(preCheck, 'preCheck !== true');
assert.sameValue(obj.hasOwnProperty("prop"), false, 'obj.hasOwnProperty("prop")');
