// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-13
description: >
    Object.create - argument 'Properties' is the JSON object (15.2.3.7
    step 2)
---*/

var result = false;

Object.defineProperty(JSON, "prop", {
  get: function() {
    result = (this === JSON);
    return {};
  },
  enumerable: true,
  configurable: true
});

var newObj = Object.create({}, JSON);

assert(result, 'result !== true');
assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
