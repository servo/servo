// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.fill
description: >
  Returns `this`.
info: |
  12. Return O.
---*/

var arr = [];
var result = arr.fill(1);

assert.sameValue(result, arr);

var o = {
  length: 0
};
result = Array.prototype.fill.call(o);
assert.sameValue(result, o);
