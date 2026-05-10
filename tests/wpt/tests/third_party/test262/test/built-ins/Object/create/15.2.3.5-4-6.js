// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-6
description: >
    Object.create - argument 'Properties' is an Array object (15.2.3.7
    step 2).
---*/

var props = [];
var result = false;

Object.defineProperty(props, "prop", {
  get: function() {
    result = this instanceof Array;
    return {};
  },
  enumerable: true
});
Object.create({}, props);

assert(result, 'result !== true');
