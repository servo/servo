// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-set-p-v-receiver
description: >
  Throws if trap is not callable (honoring the Realm of the current execution
  context)
info: |
    [[Set]] ( P, V, Receiver)

    6. Let trap be GetMethod(handler, "set").
    ...

    7.3.9 GetMethod (O, P)

    5. If IsCallable(func) is false, throw a TypeError exception.
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var p = new OProxy({}, {
  set: {}
});

assert.throws(TypeError, function() {
  p.attr = 1;
});

assert.throws(TypeError, function() {
  p["attr"] = 1;
});
