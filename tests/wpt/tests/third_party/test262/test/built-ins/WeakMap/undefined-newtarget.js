// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap-iterable
description: >
  Throws a TypeError if NewTarget is undefined.
info: |
  23.3.1.1 WeakMap ( [ iterable ] )

  1. If NewTarget is undefined, throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  WeakMap();
});

assert.throws(TypeError, function() {
  WeakMap([]);
});
