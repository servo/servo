// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.1-1_41
description: >
    Properties - [[HasOwnProperty]] (configurable, enumerable
    inherited getter property)
---*/

var base = {};
Object.defineProperty(base, "foo", {
  get: function() {
    return 42;
  },
  enumerable: true,
  configurable: true
});
var o = Object.create(base);

assert.sameValue(o.hasOwnProperty("foo"), false, 'o.hasOwnProperty("foo")');
