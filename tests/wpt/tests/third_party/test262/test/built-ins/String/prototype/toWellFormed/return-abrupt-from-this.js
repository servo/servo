// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.towellformed
description: >
  Return abrupt from RequireObjectCoercible(this value).
info: |
  String.prototype.toWellFormed( )

  1. Let O be ? RequireObjectCoercible(this value).

features: [String.prototype.toWellFormed]
---*/
assert.sameValue(typeof String.prototype.toWellFormed, 'function');

assert.throws(TypeError, function () {
  String.prototype.toWellFormed.call(undefined);
});

assert.throws(TypeError, function () {
  String.prototype.toWellFormed.call(null);
});
