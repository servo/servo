// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-3-5
description: >
    Object.defineProperties - own accessor property of 'Properties'
    which is not enumerable is not defined in 'O'
---*/

var obj = {};

var props = {};

Object.defineProperty(props, "prop", {
  get: function() {
    return {};
  },
  enumerable: false
});

Object.defineProperties(obj, props);

assert.sameValue(obj.hasOwnProperty("prop"), false, 'obj.hasOwnProperty("prop")');
