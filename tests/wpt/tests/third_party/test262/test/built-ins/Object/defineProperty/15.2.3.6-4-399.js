// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-399
description: >
    ES5 Attributes - [[Value]] attribute of data property is the
    global object
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: this
});

var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert.sameValue(obj.prop, this, 'obj.prop');
assert.sameValue(desc.value, this, 'desc.value');
