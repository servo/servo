// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  Return abrupt after getting `set` method.
info: |
  23.3.1.1 WeakMap ( [ iterable ] )

  ...
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  7. Else,
    a. Let adder be Get(map, "set").
    b. ReturnIfAbrupt(adder).
    ...
---*/

Object.defineProperty(WeakMap.prototype, 'set', {
  get: function() {
    throw new Test262Error();
  }
});

new WeakMap();
new WeakMap(null);

assert.throws(Test262Error, function() {
  new WeakMap([]);
});
