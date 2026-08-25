// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-2-4
description: >
    Object.defineProperties - argument 'Properties' is a Boolean
    object whose primitive value is true
---*/

var obj = {};
var props = new Boolean(true);
var result = false;

Object.defineProperty(props, "prop", {
  get: function() {
    result = this instanceof Boolean;
    return {};
  },
  enumerable: true
});

Object.defineProperties(obj, props);

assert(result, 'result !== true');
