// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    Trap is not callable.
info: |
    [[OwnPropertyKeys]] ( )

    5. Let trap be ? GetMethod(handler, "ownKeys").
    ...

    #sec-getmethod
    7.3.9 GetMethod (O, P)

    4. If IsCallable(func) is false, throw a TypeError exception.
features: [Proxy]
---*/

var p = new Proxy({
  attr: 1
}, {
  ownKeys: {}
});

assert.throws(TypeError, function() {
  Object.keys(p);
});
