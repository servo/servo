// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.7
description: >
    The result of [[HasProperty]] is a Boolean value and will affect has
    checkings. False returned when target property exists;
info: |
    [[HasProperty]] (P)

    ...
    12. Return booleanTrapResult.
flags: [noStrict]
features: [Proxy]
---*/

var target = {
  attr: 1
};
var p = new Proxy(target, {
  has: function(t, prop) {
    return false;
  }
});

var attr = 0;
with(p) {
  assert.sameValue(attr, 0);
}
