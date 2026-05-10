// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-312
description: >
    Object.create - [[Enumerable]] is set as false if it is absent in
    accessor descriptor of one property in 'Properties' (8.12.9 step
    4.b)
---*/

var isEnumerable = false;
var newObj = Object.create({}, {
  prop: {
    set: function() {},
    get: function() {},
    configurable: true
  }
});
var hasProperty = newObj.hasOwnProperty("prop");
for (var p in newObj) {
  if (p === "prop") {
    isEnumerable = true;
  }
}

assert(hasProperty, 'hasProperty !== true');
assert.sameValue(isEnumerable, false, 'isEnumerable');
