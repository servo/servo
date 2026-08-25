// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
description: >
    [[Call]] (thisArgument, argumentsList)

    2. If handler is null, throw a TypeError exception.
features: [Proxy]
---*/


var p = Proxy.revocable(function() {}, {});

p.revoke();

assert.throws(TypeError, function() {
  p.proxy();
});
