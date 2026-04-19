// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  Throws TypeError if add is not callable on constructor call.
info: |
  23.4.1.1 WeakSet ( [ iterable ] )

  ...
  5. If iterable is not present, let iterable be undefined.
  6. If iterable is either undefined or null, let iter be undefined.
  7. Else,
    a. Let adder be Get(set, "add").
    b. ReturnIfAbrupt(adder).
    c. If IsCallable(adder) is false, throw a TypeError exception.
  ...
---*/

WeakSet.prototype.add = null;
new WeakSet();

assert.throws(TypeError, function() {
  new WeakSet([]);
});
