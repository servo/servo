// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-2-10
description: Object.defineProperties - argument 'Properties' is an Array object
---*/

var obj = {};
var props = [];
var result = false;

Object.defineProperty(props, "prop", {
  get: function() {
    result = this instanceof Array;
    return {};
  },
  enumerable: true
});

Object.defineProperties(obj, props);

assert(result, 'result !== true');
