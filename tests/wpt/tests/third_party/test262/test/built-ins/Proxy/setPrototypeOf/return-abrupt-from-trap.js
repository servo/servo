// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.2
description: >
    Trap returns abrupt.
info: |
    [[SetPrototypeOf]] (V)

    9. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, V»)).
    10. ReturnIfAbrupt(booleanTrapResult).
features: [Proxy]
---*/

var p = new Proxy({}, {
  setPrototypeOf: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Object.setPrototypeOf(p, {
    value: 1
  });
});
