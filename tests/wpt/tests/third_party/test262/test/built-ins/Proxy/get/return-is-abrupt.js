// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.8
description: >
    Trap returns abrupt.
info: |
    [[Get]] (P, Receiver)

    9. Let trapResult be Call(trap, handler, «target, P, Receiver»).
    10. ReturnIfAbrupt(trapResult).
features: [Proxy]
---*/

var p = new Proxy({}, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  p.attr;
});

assert.throws(Test262Error, function() {
  p["attr"];
});
