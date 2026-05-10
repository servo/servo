// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
    [[Construct]] ( argumentsList, newTarget)

    2. If handler is null, throw a TypeError exception.
features: [Proxy]
---*/


var p = Proxy.revocable(function() {}, {});

p.revoke();

assert.throws(TypeError, function() {
  new p.proxy();
});
