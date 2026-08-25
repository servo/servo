// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.iswellformed
description: >
  Return abrupt from RequireObjectCoercible(this value).
info: |
  String.prototype.isWellFormed( )

  1. Let O be ? RequireObjectCoercible(this value).

features: [String.prototype.isWellFormed]
---*/
assert.sameValue(typeof String.prototype.isWellFormed, 'function');

assert.throws(TypeError, function () {
  String.prototype.isWellFormed.call(undefined);
});

assert.throws(TypeError, function () {
  String.prototype.isWellFormed.call(null);
});
