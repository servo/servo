// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-233
description: >
    Object.create - 'get' property of one property in 'Properties' is
    own data property (8.10.5 step 7.a)
---*/

var newObj = Object.create({}, {
  prop: {
    get: function() {
      return "ownDataProperty";
    }
  }
});

assert.sameValue(newObj.prop, "ownDataProperty", 'newObj.prop');
