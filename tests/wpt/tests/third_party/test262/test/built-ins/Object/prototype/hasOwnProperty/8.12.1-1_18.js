// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.12.1-1_18
description: >
    Properties - [[HasOwnProperty]] (writable, configurable,
    non-enumerable inherited value property)
---*/

var base = {};
Object.defineProperty(base, "foo", {
  value: 42,
  writable: true,
  configurable: true
});
var o = Object.create(base);

assert.sameValue(o.hasOwnProperty("foo"), false, 'o.hasOwnProperty("foo")');
