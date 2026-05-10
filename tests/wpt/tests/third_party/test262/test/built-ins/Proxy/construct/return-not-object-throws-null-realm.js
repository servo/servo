// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
  Throws a TypeError if trap result is not an Object: null
  (honoring the Realm of the current execution context)
info: |
  [[Construct]] (argumentsList, newTarget)

  [...]
  11. If Type(newObj) is not Object, throw a TypeError exception.
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var P = new OProxy(function() {
  throw new Test262Error('target should not be called');
}, {
  construct: function() {
    return null;
  },
});

assert.throws(TypeError, function() {
  new P();
});
