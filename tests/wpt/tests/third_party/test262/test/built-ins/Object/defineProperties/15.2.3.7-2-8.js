// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-2-8
description: >
    Object.defineProperties - argument 'Properties' is a String object
    whose primitive value is any interesting string
---*/

var obj = {};
var props = new String();
var result = false;

Object.defineProperty(props, "prop", {
  get: function() {
    result = this instanceof String;
    return {};
  },
  enumerable: true
});

Object.defineProperties(obj, props);

assert(result, 'result !== true');
