// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.10
description: >
    [[Delete]] (P)

    The result is a Boolean value.
features: [Proxy, Reflect]
---*/

var target = {};
var p = new Proxy(target, {
  deleteProperty: function() {
    return 0;
  }
});

Object.defineProperties(target, {
  isConfigurable: {
    value: 1,
    configurable: true
  },
  notConfigurable: {
    value: 1,
    configurable: false
  }
});

assert.sameValue(Reflect.deleteProperty(p, "attr"), false);
assert.sameValue(Reflect.deleteProperty(p, "isConfigurable"), false);
assert.sameValue(Reflect.deleteProperty(p, "notConfigurable"), false);
