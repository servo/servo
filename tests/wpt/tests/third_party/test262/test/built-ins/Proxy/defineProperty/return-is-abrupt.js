// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.6
description: >
    Trap return is an abrupt.
info: |
    [[DefineOwnProperty]] (P, Desc)

    ...
    10. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, P,
    descObj»)).
    11. ReturnIfAbrupt(booleanTrapResult).
    ...
features: [Proxy]
---*/

var p = new Proxy({}, {
  defineProperty: function(t, prop, desc) {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Object.defineProperty(p, "foo", {});
});
