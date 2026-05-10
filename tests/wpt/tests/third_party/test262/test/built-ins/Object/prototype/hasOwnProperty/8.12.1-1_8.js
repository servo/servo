// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.1-1_8
description: >
    Properties - [[HasOwnProperty]] (non-writable, configurable,
    enumerable own value property)
---*/

var o = {};
Object.defineProperty(o, "foo", {
  value: 42,
  configurable: true,
  enumerable: true
});

assert(o.hasOwnProperty("foo"), 'o.hasOwnProperty("foo") !== true');
