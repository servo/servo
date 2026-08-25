// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-9
description: >
    Object.create - argument 'Properties' is a Number object whose
    primitive value is any interesting number (15.2.3.7 step 2).
---*/

var props = new Number(12);
var result = false;

Object.defineProperty(props, "prop", {
  get: function() {
    result = this instanceof Number;
    return {};
  },
  enumerable: true
});
Object.create({}, props);

assert(result, 'result !== true');
