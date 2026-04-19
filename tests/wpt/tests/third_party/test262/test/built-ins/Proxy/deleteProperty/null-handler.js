// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.10
description: >
    [[Delete]] (P)

    3. If handler is null, throw a TypeError exception.
features: [Proxy]
---*/

var p = Proxy.revocable({
  attr: 1
}, {});

p.revoke();

assert.throws(TypeError, function() {
  delete p.proxy.attr;
});
