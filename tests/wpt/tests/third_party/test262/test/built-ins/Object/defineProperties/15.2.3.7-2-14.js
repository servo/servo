// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-2-14
description: Object.defineProperties - argument 'Properties' is the JSON object
---*/

var obj = {};
var result = false;

Object.defineProperty(JSON, "prop", {
  get: function() {
    result = (this === JSON);
    return {};
  },
  enumerable: true,
  configurable: true
});

Object.defineProperties(obj, JSON);

assert(result, 'result !== true');
