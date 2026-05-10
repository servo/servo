// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    [[OwnPropertyKeys]] ( )

    2. If handler is null, throw a TypeError exception.
features: [Proxy]
---*/

var p = Proxy.revocable({}, {});

p.revoke();

assert.throws(TypeError, function() {
  Object.keys(p.proxy);
});
