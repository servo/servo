// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializers-in-forin-statement-heads
description: >
    for-in initializers in nonstrict mode
flags: [noStrict]
---*/
(function() {
  var effects = 0;
  for (var a = ++effects in {});
  assert.sameValue(effects, 1);
})();


(function() {
  var stored;
  for (var a = 0 in stored = a, {});
  assert.sameValue(stored, 0);
})();


(function() {
  for (var a = 0 in {});
  assert.sameValue(a, 0);
})();


(function() {
  var effects = 0;
  var iterations = 0;
  var stored;
  for (var a = (++effects, -1) in stored = a, {a: 0, b: 1, c: 2}) {
    ++iterations;
  }
  assert.sameValue(stored, -1, "Initialized value should be available to RHS");
  assert.sameValue(effects, 1, "Initializer should only be executed once");
  assert.sameValue(iterations, 3, "Loop body should be executed the appropriate number of times");
})();
