// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  Throws a TypeError if Map is called without a newTarget.
info: |
  Map ( [ iterable ] )

  When the Map function is called with optional argument the following steps
  are taken:

  1. If NewTarget is undefined, throw a TypeError exception.
  ...

---*/

assert.throws(TypeError, function() {
  Map();
});

assert.throws(TypeError, function() {
  Map([]);
});
