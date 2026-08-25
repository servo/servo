// Copyright (C) 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.tostring
description: >
  toString of Proxy for non-callable target throws
info: |
  ...
  Throw a TypeError exception.

features: [Proxy]
---*/

assert.throws(TypeError, function() {
  Function.prototype.toString.call(new Proxy({}, {}));
});
