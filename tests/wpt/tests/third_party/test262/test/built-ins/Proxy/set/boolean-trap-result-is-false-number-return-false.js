// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.9
description: >
    [[Set]] ( P, V, Receiver)

    11. If booleanTrapResult is false, return false.
features: [Proxy, Reflect, Reflect.set]
---*/

var target = {};
var handler = {
  set: function(t, prop, value, receiver) {
    return 0;
  }
};
var p = new Proxy(target, handler);

assert.sameValue(Reflect.set(p, "attr", "foo"), false);
