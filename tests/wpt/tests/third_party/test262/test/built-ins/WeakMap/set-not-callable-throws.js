// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  Throws TypeError if add is not callable on constructor call.
info: |
  23.3.1.1 WeakMap ( [ iterable ] )

  ...
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  7. Else,
    a. Let adder be Get(map, "set").
    b. ReturnIfAbrupt(adder).
    c. If IsCallable(adder) is false, throw a TypeError exception.
  ...
---*/

WeakMap.prototype.set = null;
new WeakMap();

assert.throws(TypeError, function() {
  new WeakMap([]);
});
