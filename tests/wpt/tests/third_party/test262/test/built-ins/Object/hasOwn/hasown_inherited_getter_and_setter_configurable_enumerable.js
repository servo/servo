// Copyright (c) 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
description: >
    Properties - [[HasOwnProperty]] (configurable, enumerable
    inherited getter/setter property)
author: Jamie Kyle
features: [Object.hasOwn]
---*/

var base = {};
Object.defineProperty(base, "foo", {
  get: function() {
    return 42;
  },
  set: function() {;
  },
  enumerable: true,
  configurable: true
});
var o = Object.create(base);

assert.sameValue(Object.hasOwn(o, "foo"), false, 'Object.hasOwn(o, "foo")');
