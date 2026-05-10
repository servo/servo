// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  Throws TypeError when `this` is undefined
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  1. Let O be RequireObjectCoercible(this value).
  2. Let S be ToString(O).
---*/

assert.throws(TypeError, function() {
  String.prototype.startsWith.call(undefined, '');
});
