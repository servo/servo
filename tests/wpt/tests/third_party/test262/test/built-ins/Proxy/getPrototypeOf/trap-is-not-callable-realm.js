// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-getprototypeof
description: >
  Throws if trap is not callable (honoring the Realm of the current execution
  context)
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var p = new OProxy({}, {
  getPrototypeOf: {}
});

assert.throws(TypeError, function() {
  Object.getPrototypeOf(p);
});
