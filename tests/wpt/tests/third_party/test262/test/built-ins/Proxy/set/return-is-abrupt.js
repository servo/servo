// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.9
description: >
    Trap returns abrupt.
info: |
    [[Set]] ( P, V, Receiver)

    ...
    9. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target, P, V, Receiver»)).
    10. ReturnIfAbrupt(booleanTrapResult).
    ...
features: [Proxy]
---*/

var p = new Proxy({}, {
  set: function(t, prop, value, receiver) {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  p.attr = "bar";
});

assert.throws(Test262Error, function() {
  p["attr"] = "bar";
});
