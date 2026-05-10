// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
description: >
    Throws if trap is not callable.
features: [Proxy]
---*/

var p = new Proxy(function() {}, {
  apply: {}
});

assert.throws(TypeError, function() {
  p();
});
