// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset-iterable
description: >
  The WeakSet constructor is the %WeakSet% intrinsic object and the initial
  value of the WeakSet property of the global object.
info: |
  23.4.1.1 WeakSet ( [ iterable ] )

  1. If NewTarget is undefined, throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  WeakSet();
});

assert.throws(TypeError, function() {
  WeakSet([]);
});
