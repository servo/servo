// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: Abrupt completion from `ToString` operation (Symbol value)
info: |
    1. Let string be ? ToString(string).
features: [Symbol]
---*/

var s = Symbol('');

assert.throws(TypeError, function() {
  unescape(s);
});
