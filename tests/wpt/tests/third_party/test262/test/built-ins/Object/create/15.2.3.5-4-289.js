// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-289
description: >
    Object.create - one property in 'Properties' is an Arguments
    object which implements its own [[Get]] method to access the 'set'
    property (8.10.5 step 8.a)
---*/

var argObj = (function() {
  return arguments;
})();

var data = "data";

argObj.set = function(value) {
  data = value;
};

var newobj = Object.create({}, {
  prop: argObj
});

var hasProperty = newobj.hasOwnProperty("prop");

newobj.prop = "overrideData";

assert(hasProperty, 'hasProperty !== true');
assert.sameValue(data, "overrideData", 'data');
