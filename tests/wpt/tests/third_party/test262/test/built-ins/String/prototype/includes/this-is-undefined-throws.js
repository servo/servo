// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.7
description: >
  Throws TypeError when `this` is undefined
info: |
  21.1.3.7 String.prototype.includes ( searchString [ , position ] )

  1. Let O be RequireObjectCoercible(this value).
  2. Let S be ToString(O).
features: [String.prototype.includes]
---*/
assert.throws(TypeError, function() {
  String.prototype.includes.call(undefined, '');
});
