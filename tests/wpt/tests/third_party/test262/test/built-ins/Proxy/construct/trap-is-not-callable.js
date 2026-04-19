// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
    Throws if trap is not callable.
features: [Proxy]
---*/

function Target() {}
var p = new Proxy(Target, {
  construct: {}
});

assert.throws(TypeError, function() {
  new p();
});
