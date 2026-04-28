// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-a-15
description: >
    Object.defineProperties - 'Properties' is the JSON object which
    implements its own [[Get]] method to get enumerable own property
---*/

var obj = {};

JSON.prop = {
  value: 15
};
Object.defineProperties(obj, JSON);

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 15, 'obj.prop');
