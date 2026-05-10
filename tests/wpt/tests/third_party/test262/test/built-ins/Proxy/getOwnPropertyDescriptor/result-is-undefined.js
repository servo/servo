// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Return undefined if trap result is undefined and target is extensible and
    the target property descriptor is configurable.
info: |
    [[GetOwnProperty]] (P)

    ...
    14. If trapResultObj is undefined, then
        ...
        f. Return undefined.
    ...
features: [Proxy]
---*/

var target = {
  attr: 1
};

var p = new Proxy(target, {
  getOwnPropertyDescriptor: function(t, prop) {
    return;
  }
});

assert.sameValue(Object.getOwnPropertyDescriptor(p, "attr"), undefined);
