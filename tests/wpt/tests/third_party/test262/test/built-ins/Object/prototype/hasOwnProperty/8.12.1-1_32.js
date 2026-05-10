// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.1-1_32
description: >
    Properties - [[HasOwnProperty]] (configurable, non-enumerable own
    setter property)
---*/

var o = {};
Object.defineProperty(o, "foo", {
  set: function() {;
  },
  configurable: true
});

assert(o.hasOwnProperty("foo"), 'o.hasOwnProperty("foo") !== true');
